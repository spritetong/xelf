#[path = "orm_utils.rs"]
mod orm_utils;

use crate::prelude::*;
use orm_utils::*;

pub use sea_orm::{
    entity::prelude::*,
    sea_query::{ConditionExpression, Expr, Func, Query, SimpleExpr},
    Condition, ConnectOptions, ConnectionTrait, Database, DatabaseBackend,
    DatabaseTransaction, DbBackend, DbErr, ExecResult, FromQueryResult, IntoActiveModel,
    JoinType, NotSet, QueryOrder, QuerySelect, QueryTrait, Set, Statement,
    TransactionTrait, Unchanged,
};

pub type DbResult<T> = Result<T, DbErr>;

////////////////////////////////////////////////////////////////////////////////

pub trait OrmModelExt {
    fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;
}

pub trait OrmActiveModelExt {
    fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;
}

macro_rules! impl_update_by_json {
    () => {
        fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
        where
            S: ?Sized + Contains<C, str>,
            C: Eq + Ord + Hash + Borrow<str>,
        {
            let map = some_or_return!(
                jsn.as_object(),
                Err(DbErr::Type("Invalid JSON object".to_owned()))
            );

            orm_validate_json::<M, S, C>(map, skip)?;

            for c in E::Column::iter() {
                if !skip.contains_it(&c.as_str()) {
                    if let Some(v) = map.get(c.as_str()) {
                        match orm_json_to_db_value(c.def().get_column_type(), v) {
                            Some(v) => {
                                self.set(c, v);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        }
    };
}

impl<E, M> OrmModelExt for M
where
    M: ModelTrait<Entity = E> + Default + Serialize + DeserializeOwned,
    E: EntityTrait<Model = M>,
{
    impl_update_by_json!();
}

impl<E, M, A> OrmActiveModelExt for A
where
    A: ActiveModelTrait<Entity = E>,
    E: EntityTrait<Model = M>,
    M: ModelTrait<Entity = E> + Default + Serialize + DeserializeOwned,
{
    impl_update_by_json!();
}

////////////////////////////////////////////////////////////////////////////////

type ParamIndices = smallvec::SmallVec<[u16; 4]>;
type ParamMap = LinkedHashMap<ByteString, ParamIndices>;

#[derive(Deref, DerefMut, Clone, Debug)]
pub struct SqlHelper {
    #[deref]
    #[deref_mut]
    statement: Statement,
    params: Arc<ParamMap>,
}

impl SqlHelper {
    pub fn into_statement(self) -> Statement {
        self.statement
    }

    pub fn iter_params(&self) -> SqlParamIterator {
        let params = self.params.clone();
        SqlParamIteratorBuilder {
            params,
            it_builder: |x| x.iter(),
        }
        .build()
    }

    pub fn bind_param<N: AsRef<str>, V: Into<Value>>(&mut self, name: N, value: V) -> &mut Self {
        let indices = self.params.get(name.as_ref()).unwrap();
        let value = value.into();
        if let Some(ref mut values) = self.statement.values {
            if indices.len() == 1 {
                values.0[*indices.first().unwrap() as usize] = value;
            } else {
                for &i in indices {
                    values.0[i as usize] = value.clone();
                }
            }
        }
        self
    }

    #[inline]
    pub fn bind_optional<N: AsRef<str>>(&mut self, name: N, optional: bool) -> &mut Self {
        self.bind_param(name, optional as i32)
    }
}

impl From<Statement> for SqlHelper {
    fn from(statement: Statement) -> Self {
        let mut params = ParamMap::new();
        if let Some(values) = &statement.values {
            for (index, param) in values.iter().enumerate() {
                if let Value::String(Some(name)) = param {
                    if name.starts_with(':') {
                        match params.get_mut(name.as_str()) {
                            Some(v) => v.push(index as u16),
                            _ => {
                                let mut indices = ParamIndices::new();
                                indices.push(index as u16);
                                params.insert(name.deref().clone().into(), indices);
                            }
                        }
                    }
                }
            }
        }
        //println!("{:?} {:?}", &statement, &params);
        Self {
            statement,
            params: Arc::new(params),
        }
    }
}

impl Into<Statement> for SqlHelper {
    fn into(self) -> Statement {
        self.statement
    }
}

#[self_referencing]
pub struct SqlParamIterator {
    params: Arc<ParamMap>,
    #[borrows(params)]
    #[covariant]
    it: ritelinked::linked_hash_map::Iter<'this, ByteString, ParamIndices>,
}

impl Iterator for SqlParamIterator {
    type Item = ByteString;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.with_it_mut(|x| x.next().map(|x| x.0.clone()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.borrow_it().size_hint()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DbLockMode {
    /// Protects a table from concurrent data changes. (SQLite: BEGIN)
    Share,
    /// Do not allow other data changes. (SQLite: BEGIN IMMEDIATE)
    Exclusive,
    /// Do not allow any other access. (SQLite: BEGIN EXCLUSIVE)
    AccessExclusive,
}

#[derive(Clone, Deref, Debug)]
pub struct IdenStr<T: AsRef<str> + Clone + Send + Sync>(T);

impl<T: AsRef<str> + Clone + Send + Sync> Iden for IdenStr<T> {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        let _ = s.write_str(self.0.as_ref());
    }
}

#[async_trait]
pub trait DbConnExt: ConnectionTrait {
    fn builtin_func(&self, name: &str) -> &'static str {
        match self.get_database_backend() {
            DbBackend::Postgres => match name {
                "now()" => return "now()",
                "least" => return "least",
                "greatest" => return "greatest",
                "upper" => return "upper",
                "lower" => return "lower",
                _ => (),
            },
            DbBackend::Sqlite => match name {
                "now()" => return "strftime('%Y-%m-%d %H:%M:%f000000', DATETIME('now'))",
                "least" => return "min",
                "greatest" => return "max",
                "upper" => return "upper",
                "lower" => return "lower",
                _ => (),
            },
            _ => (),
        }

        panic!(
            "No built-in function {} for {:?}",
            name,
            self.get_database_backend()
        );
    }

    fn func(&self, name: &str) -> SimpleExpr {
        Expr::cust(self.builtin_func(name))
    }

    fn func_unary<T>(&self, name: &str, arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Func::cust(IdenStr(self.builtin_func(name))).args(vec![T::into(arg)])
    }

    fn func_with_args<T, I>(&self, name: &str, args: I) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
        I: IntoIterator<Item = T>,
    {
        Func::cust(IdenStr(self.builtin_func(name))).args(args)
    }

    async fn lock_table(&self, table: &str, mode: DbLockMode) -> DbResult<()> {
        match self.get_database_backend() {
            DbBackend::Postgres => {
                if !table.is_empty() {
                    let mode = match mode {
                        DbLockMode::Share => "SHARE",
                        DbLockMode::Exclusive => "EXCLUSIVE",
                        DbLockMode::AccessExclusive => "ACCESS EXCLUSIVE",
                    };
                    let sql = format!("LOCK TABLE {} IN {} MODE;", table, mode);
                    self.execute(Statement::from_string(self.get_database_backend(), sql))
                        .await?;
                }
            }
            DbBackend::Sqlite => (),
            _ => return Err(DbErr::Custom("no implementation".to_owned())),
        }
        Ok(())
    }
}

#[async_trait]
impl<C: ConnectionTrait> DbConnExt for C {}

pub fn db_optional<P, C>(param: P, condition: C) -> Condition
where
    P: Into<Value>,
    C: Into<ConditionExpression>,
{
    Condition::any()
        .add(Expr::cust_with_values("1 = ?", vec![P::into(param)]))
        .add(C::into(condition))
}

////////////////////////////////////////////////////////////////////////////////

struct OrderByField {
    field: String,
    asc: bool,
    wrapper_func: IdenStr<ByteString>,
}

pub struct QueryHelper {
    order_fields: Vec<OrderByField>,
}

impl QueryHelper {
    pub fn new<C, F>(order_by: Option<&Json>, wrapper_funcs: Option<&BTreeMap<C, F>>) -> Self
    where
        C: Ord + Borrow<str>,
        F: Ord + AsRef<str>,
    {
        let mut order_fields: Vec<OrderByField> = vec![];

        match order_by {
            Some(v) if v.is_string() => {
                let order_by = v.as_str().unwrap();
                let re =
                    Regex::new(r"\b\s*([[:word:]]+)\s*((?i:ASC|DESC)?)\s*\b(?:,|;|$)").unwrap();
                for cap in re.captures_iter(order_by) {
                    order_fields.push(OrderByField {
                        field: cap[1].to_owned(),
                        asc: cap[2].to_ascii_uppercase() != "DESC",
                        wrapper_func: IdenStr(
                            wrapper_funcs
                                .and_then(|x| x.get(&cap[1]).map(|x| x.as_ref().to_owned()))
                                .unwrap_or_else(|| String::new())
                                .into(),
                        ),
                    });
                }
            }
            _ => (),
        }

        Self { order_fields }
    }

    pub fn query<E, M>(
        &mut self,
        entity: E,
        mut select: Select<E>,
        after: Option<&Json>,
        id_field: Option<&str>,
    ) -> Select<E>
    where
        E: EntityTrait<Model = M> + sea_orm::sea_query::IntoIden,
        M: ModelTrait<Entity = E> + Default + OrmModelExt,
    {
        if let Some(jsn @ &Json::Object(_)) = after {
            let mut model = M::default();
            if model.update_by_json(jsn, &None::<&str>).is_ok() {
                // "<id_field>" <> after.<id_field>
                let id_field = id_field.unwrap_or("id");
                let id_col_name = id_field
                    .split('.')
                    .into_iter()
                    .rev()
                    .next()
                    .unwrap_or(id_field);
                if let Ok(id_col) = E::Column::from_str(id_col_name) {
                    let after_id = model.get(id_col);
                    if !sea_orm::sea_query::sea_value_to_json_value(&after_id).is_null() {
                        select =
                            QueryFilter::filter(select, Expr::tbl(entity, id_col).ne(after_id));
                    }
                }

                for pat in self.order_fields.iter() {
                    if let Ok(col) = E::Column::from_str(&pat.field) {
                        match jsn.get(&pat.field) {
                            None | Some(Json::Null) => (),
                            _ => {
                                let mut field = Expr::tbl(entity, col);
                                let mut value = Expr::val(model.get(col));
                                if !pat.wrapper_func.is_empty() {
                                    field = Expr::expr(
                                        Func::cust(pat.wrapper_func.clone()).args([field]),
                                    );
                                    value = Expr::expr(
                                        Func::cust(pat.wrapper_func.clone()).args([value]),
                                    )
                                }
                                if pat.asc {
                                    select =
                                        QueryFilter::filter(select, field.greater_or_equal(value));
                                } else {
                                    select =
                                        QueryFilter::filter(select, field.less_or_equal(value));
                                }
                            }
                        }
                    }
                }
            }
        }

        for pat in self.order_fields.iter() {
            if let Ok(col) = E::Column::from_str(&pat.field) {
                let mut field = Expr::tbl(entity, col).into_simple_expr();
                if !pat.wrapper_func.is_empty() {
                    field = Func::cust(pat.wrapper_func.clone()).args([field]);
                }
                if pat.asc {
                    select = select.order_by_asc(field);
                } else {
                    select = select.order_by_desc(field);
                }
            }
        }

        select
    }

    pub fn query_with_args<E, M, C, F>(
        entity: E,
        select: Select<E>,
        args: &JsonMap,
        id_field: Option<&str>,
        wrapper_funcs: Option<&BTreeMap<C, F>>,
    ) -> Select<E>
    where
        E: EntityTrait<Model = M> + sea_orm::sea_query::IntoIden,
        M: ModelTrait<Entity = E> + Default + OrmModelExt,
        C: Ord + Borrow<str>,
        F: Ord + AsRef<str>,
    {
        let mut helper = Self::new(args.get("order_by$"), wrapper_funcs);
        helper.query(entity, select, args.get("after$"), id_field)
    }
}

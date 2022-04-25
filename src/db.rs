#[path = "orm_utils.rs"]
mod orm_utils;

use crate::prelude::*;
use orm_utils::*;

pub use sea_orm::{
    entity::prelude::*,
    sea_query::{
        BinOper, ConditionExpression, DynIden, Expr, Func, IntoIden, JoinOn, LogicalChainOper,
        Query, QueryBuilder, SimpleExpr, SqlWriter, UnOper,
    },
    Condition, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseTransaction,
    DbBackend, DbErr, ExecResult, FromQueryResult, IntoActiveModel, JoinType, NotSet, Order,
    QueryOrder, QuerySelect, QueryTrait, SelectGetableValue, SelectModel, SelectTwoModel,
    SelectorRaw, Set, Statement, StreamTrait, TransactionTrait, Unchanged, Values, ActiveValue
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DbLockMode {
    /// Protects a table from concurrent data changes. (SQLite: BEGIN)
    Share,
    /// Do not allow other data changes. (SQLite: BEGIN IMMEDIATE)
    Exclusive,
    /// Do not allow any other access. (SQLite: BEGIN EXCLUSIVE)
    AccessExclusive,
}

#[derive(Clone, Deref, Debug)]
pub struct IdenStr<T: AsRef<str> + Clone + Send + Sync>(pub T);

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
        .add(Expr::cust_with_values("0 = ?", [P::into(param)]))
        .add(C::into(condition))
}

////////////////////////////////////////////////////////////////////////////////

pub struct RawSqlBuilder {
    db_backend: DbBackend,
    builder: Box<dyn QueryBuilder>,
    writer: SqlWriter,
    values: Vec<Value>,
}

impl RawSqlBuilder {
    pub fn new(db_backend: DbBackend) -> Self {
        Self {
            db_backend,
            builder: db_backend.get_query_builder(),
            writer: SqlWriter::new(),
            values: Vec::new(),
        }
    }

    pub fn into_statement(self) -> Statement {
        let Self {
            db_backend,
            writer,
            values,
            ..
        } = self;

        Statement {
            sql: writer.result(),
            values: Some(Values(values)),
            db_backend,
        }
    }

    pub fn into_sql_helper(self) -> SqlHelper {
        self.into_statement().into()
    }

    pub fn into_select<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        M::find_by_statement(self.into())
    }

    pub fn into_select_two<M, N>(self) -> SelectorRaw<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        // TODO: There's no safe methods to transmute Statement into SelectorRaw<SelectTwoModel>.
        unsafe { mem::transmute(self.into_statement()) }
    }

    pub fn into_json(self) -> SelectorRaw<SelectModel<Json>> {
        SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(self.into())
    }

    pub fn into_values<T, C>(self) -> SelectorRaw<SelectGetableValue<T, C>>
    where
        T: sea_orm::TryGetableMany,
        C: sea_orm::Iterable + sea_orm::strum::IntoEnumIterator + Iden,
    {
        SelectorRaw::<SelectGetableValue<T, C>>::with_columns::<T, C>(self.into())
    }

    #[inline]
    pub fn get_database_backend(&self) -> DbBackend {
        self.db_backend
    }

    pub fn write_expr(&mut self, expr: &SimpleExpr) {
        let mut collector = |x| self.values.push(x);
        self.builder
            .prepare_simple_expr(expr, &mut self.writer, &mut collector);
    }

    pub fn write(&mut self, s: &str) {
        self.write_expr(&Expr::cust(s));
    }

    pub fn write_with_args<V, I>(&mut self, s: &str, v: I)
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        self.write_expr(&Expr::cust_with_values(s, v));
    }

    pub fn expr_to_string(db_backend: DbBackend, expr: &SimpleExpr) -> String {
        let mut w = RawSqlBuilder::new(db_backend);
        w.write_expr(expr);
        w.into_statement().to_string()
    }
}

impl Into<Statement> for RawSqlBuilder {
    fn into(self) -> Statement {
        self.into_statement()
    }
}

impl fmt::Debug for RawSqlBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawSqlBuilder")
            .field("db_backend", &self.db_backend)
            .field("SQL", &self.writer)
            .field("values", &self.values)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParamIndex {
    Sql(u32),
    Value(u32),
}

type ParamIndices = smallvec::SmallVec<[ParamIndex; 4]>;
type ParamMap = LinkedHashMap<ByteString, ParamIndices>;

#[derive(Deref, DerefMut, Clone, Debug)]
pub struct SqlHelper {
    #[deref]
    #[deref_mut]
    statement: Statement,
    sql_slices: Vec<ByteString>,
    params: Arc<ParamMap>,
}

impl SqlHelper {
    pub fn into_statement(self) -> Statement {
        let Self {
            mut statement,
            sql_slices,
            ..
        } = self;

        if !sql_slices.is_empty() {
            let len = sql_slices.iter().fold(0usize, |n, x| n + x.len());
            statement.sql.clear();
            statement.sql.reserve(len);
            sql_slices
                .iter()
                .for_each(|x| statement.sql.write_str(x.deref()).unwrap());
        }
        statement
    }

    pub fn into_select<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        M::find_by_statement(self.into())
    }

    pub fn into_select_two<M, N>(self) -> SelectorRaw<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        // TODO: There's no safe methods to transmute Statement into SelectorRaw<SelectTwoModel>.
        unsafe { mem::transmute(self.into_statement()) }
    }

    pub fn into_json(self) -> SelectorRaw<SelectModel<Json>> {
        SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(self.into())
    }

    pub fn into_values<T, C>(self) -> SelectorRaw<SelectGetableValue<T, C>>
    where
        T: sea_orm::TryGetableMany,
        C: sea_orm::Iterable + sea_orm::strum::IntoEnumIterator + Iden,
    {
        SelectorRaw::<SelectGetableValue<T, C>>::with_columns::<T, C>(self.into())
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
        let value = value.into();
        for &idx in self.params.get(name.as_ref()).unwrap() {
            match idx {
                ParamIndex::Value(i) => {
                    if let Some(ref mut values) = self.statement.values {
                        values.0[i as usize] = value.clone();
                    }
                }
                ParamIndex::Sql(i) => {
                    if let Value::String(Some(s)) = &value {
                        self.sql_slices[i as usize] = s.deref().clone().into();
                    } else {
                        panic!(
                            "Can not set the SQL slice \"{}\" as {:?}",
                            name.as_ref(),
                            &value
                        );
                    }
                }
            }
        }
        self
    }

    #[inline]
    pub fn bind_optional<N: AsRef<str>>(&mut self, name: N, optional: bool) -> &mut Self {
        self.bind_param(name, optional as i32)
    }

    #[inline]
    pub fn expr_to_string(&self, expr: &SimpleExpr) -> String {
        RawSqlBuilder::expr_to_string(self.statement.db_backend, expr)
    }
}

impl From<Statement> for SqlHelper {
    fn from(statement: Statement) -> Self {
        let mut params = ParamMap::new();
        let mut sql_slices = Vec::new();

        // Get value indices.
        if let Some(values) = &statement.values {
            for (index, param) in values.iter().enumerate() {
                if let Value::String(Some(name)) = param {
                    if name.starts_with(':') {
                        params
                            .raw_entry_mut()
                            .from_key(name.as_str())
                            .or_insert_with(|| (name.deref().clone().into(), ParamIndices::new()))
                            .1
                            .push(ParamIndex::Value(index as u32));
                    }
                }
            }
        }

        // Get SQL block indices.
        let sql = statement.sql.as_str();
        let mut left = sql.as_bytes();
        let _ = shellexpand::env_with_context_no_errors::<str, &str, _>(sql, |name| {
            if name.starts_with(':') {
                let (a, b) = left.split_at((name.as_ptr() as usize) - (left.as_ptr() as usize) - 2);
                // Push SQL text before the parameter.
                sql_slices.push(unsafe { mem::transmute::<_, &str>(a).to_owned().into() });

                let (a, b) = b.split_at(name.len() + 3);
                // Push the parameter: ${<name>}
                sql_slices.push(unsafe { mem::transmute::<_, &str>(a).to_owned().into() });

                // Save the left slice.
                left = b;

                params
                    .raw_entry_mut()
                    .from_key(name)
                    .or_insert_with(|| (name.deref().clone().into(), ParamIndices::new()))
                    .1
                    .push(ParamIndex::Sql((sql_slices.len() - 1) as u32));
            }
            None
        });
        if !sql_slices.is_empty() {
            sql_slices.push(unsafe { mem::transmute::<_, &str>(left).to_owned().into() });
        }

        //println!("{:?} {:?}", &statement, &params);
        Self {
            statement,
            sql_slices,
            params: Arc::new(params),
        }
    }
}

impl Into<Statement> for SqlHelper {
    fn into(self) -> Statement {
        self.into_statement()
    }
}

impl From<RawSqlBuilder> for SqlHelper {
    fn from(builder: RawSqlBuilder) -> Self {
        Into::<Statement>::into(builder).into()
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

struct OrderByField {
    field: String,
    asc: bool,
    wrapper_func: IdenStr<ByteString>,
    aggregate_func: IdenStr<ByteString>,
}

pub struct QueryHelper {
    entity: DynIden,
    id_field: String,
    order_by: Vec<OrderByField>,
}

impl QueryHelper {
    pub fn new<T>(entity: T) -> Self
    where
        T: IntoIden,
    {
        Self {
            entity: entity.into_iden(),
            id_field: "id".to_owned(),
            order_by: Vec::new(),
        }
    }

    pub fn set_id_field<T>(&mut self, id_field: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.id_field = id_field.as_ref().to_owned();
        self
    }

    pub fn set_order_by<C, F>(
        &mut self,
        order_by: Option<&Json>,
        wrapper_funcs: Option<&HashMap<C, F>>,
        aggregate_funcs: Option<&HashMap<C, F>>,
    ) -> &mut Self
    where
        C: Hash + Eq + Borrow<str>,
        F: Hash + Eq + AsRef<str>,
    {
        self.order_by.clear();
        if let Some(Json::String(order_by)) = order_by {
            let re = Regex::new(r"\b\s*([[:word:]]+)\s*((?i:ASC|DESC)?)\s*\b(?:,|;|$)").unwrap();
            for cap in re.captures_iter(order_by) {
                self.order_by.push(OrderByField {
                    field: cap[1].to_owned(),
                    asc: cap[2].to_ascii_uppercase() != "DESC",
                    wrapper_func: IdenStr(
                        wrapper_funcs
                            .and_then(|x| x.get(&cap[1]).map(|x| x.as_ref().to_owned()))
                            .unwrap_or_else(|| String::new())
                            .into(),
                    ),
                    aggregate_func: IdenStr(
                        aggregate_funcs
                            .and_then(|x| x.get(&cap[1]).map(|x| x.as_ref().to_owned()))
                            .unwrap_or_else(|| String::new())
                            .into(),
                    ),
                });
            }
        }
        self
    }

    pub fn write_filters<E>(&self, after: Option<&Json>, writer: &mut dyn FnMut(SimpleExpr))
    where
        E: EntityTrait,
        E::Model: Default + OrmModelExt,
    {
        // filters
        if let Some(jsn @ &Json::Object(_)) = after {
            let mut model = E::Model::default();
            if model.update_by_json(jsn, &None::<&str>).is_ok() {
                // "<id_field>" <> after.<id_field>
                let id_col_name = self
                    .id_field
                    .split('.')
                    .into_iter()
                    .rev()
                    .next()
                    .unwrap_or(self.id_field.as_str());
                if let Ok(id_col) = E::Column::from_str(id_col_name) {
                    let after_id = model.get(id_col);
                    if !sea_orm::sea_query::sea_value_to_json_value(&after_id).is_null() {
                        writer(Expr::tbl(self.entity.clone(), id_col).ne(after_id));
                    }
                }

                for pat in self.order_by.iter() {
                    if let Ok(col) = E::Column::from_str(&pat.field) {
                        match jsn.get(&pat.field) {
                            None | Some(Json::Null) => (),
                            _ => {
                                let mut field = Expr::tbl(self.entity.clone(), col);
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
                                    writer(field.greater_or_equal(value));
                                } else {
                                    writer(field.less_or_equal(value));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn write_order_by<E>(&self, writer: &mut dyn FnMut(SimpleExpr, Order))
    where
        E: EntityTrait,
    {
        for pat in self.order_by.iter() {
            if let Ok(col) = E::Column::from_str(&pat.field) {
                let mut field = Expr::tbl(self.entity.clone(), col).into_simple_expr();
                if !pat.wrapper_func.is_empty() {
                    field = Func::cust(pat.wrapper_func.clone()).args([field]);
                }
                if !pat.aggregate_func.is_empty() {
                    field = Func::cust(pat.aggregate_func.clone()).args([field]);
                }
                if pat.asc {
                    writer(field, Order::Asc);
                } else {
                    writer(field, Order::Desc);
                }
            }
        }
    }

    pub fn select_filters<E>(&self, select: Select<E>, after: Option<&Json>) -> Select<E>
    where
        E: EntityTrait,
        E::Model: Default + OrmModelExt,
    {
        let mut select = Some(select);
        let mut writer = |x| {
            select = Some(Option::take(&mut select).unwrap().filter(x));
        };
        self.write_filters::<E>(after, &mut writer);
        select.unwrap()
    }

    pub fn select_order_by<E>(&self, select: Select<E>) -> Select<E>
    where
        E: EntityTrait,
    {
        let mut select = Some(select);
        let mut writer = |x, order| {
            select = Some(Option::take(&mut select).unwrap().order_by(x, order));
        };
        self.write_order_by::<E>(&mut writer);
        select.unwrap()
    }

    pub fn raw_sql_filters<E>(&self, builder: &mut RawSqlBuilder, after: Option<&Json>)
    where
        E: EntityTrait,
        E::Model: Default + OrmModelExt,
    {
        let mut writer = |x| {
            builder.write(" AND ");
            builder.write_expr(&x);
        };
        self.write_filters::<E>(after, &mut writer);
    }

    pub fn raw_sql_order_by<E>(&self, builder: &mut RawSqlBuilder)
    where
        E: EntityTrait,
    {
        let mut sep = " ORDER BY ";
        let mut writer = |x, order| {
            builder.write(sep);
            builder.write_expr(&x);
            match order {
                Order::Asc => builder.write(" ASC"),
                Order::Desc => builder.write(" DESC"),
                _ => (),
            }
            sep = ", "
        };
        self.write_order_by::<E>(&mut writer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_helper() {
        let mut w = RawSqlBuilder::new(DbBackend::Postgres);
        w.write("SELECT * FROM t_user\n");
        w.write_with_args("WHERE name = ?\n", [":name"]);
        w.write("${:order_by}\n");
        w.write("${:limit}\n");
        w.write("${:order_by}\n");
        w.write("FOR UPDATE");

        let mut q = SqlHelper::from(w);
        q.bind_param(":name", "Tom");
        q.bind_param(":order_by", "ORDER BY name");
        q.bind_param(":limit", "LIMIT 100");

        let a = Expr::expr(Expr::cust("A")).is_in(["1", "2", "3"]);
        println!("{}", q.expr_to_string(&a));
        let a = Expr::expr(Expr::cust("A")).is_in([Utc::now()]);
        println!("{}", q.expr_to_string(&a));

        let statement = q.into_statement();
        println!("{:?}", &statement);

        assert_eq!(
            &statement.sql,
            "SELECT * FROM t_user\nWHERE name = $1\nORDER BY name\nLIMIT 100\nORDER BY name\nFOR UPDATE"
        );
    }
}

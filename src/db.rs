use crate::prelude::*;

pub use sea_orm::{
    entity::prelude::*,
    sea_query::{
        BinOper, ConditionExpression, DynIden, Expr, Func, Function, IntoIden, JoinOn, LikeExpr,
        LogicalChainOper, Query, QueryBuilder, SimpleExpr, SqlWriter, UnOper,
    },
    ActiveValue, Condition, ConnectOptions, ConnectionTrait, Database, DatabaseBackend,
    DatabaseTransaction, DbBackend, DbErr, ExecResult, FromQueryResult, IntoActiveModel, JoinType,
    NotSet, Order, QueryOrder, QuerySelect, QueryTrait, SelectGetableValue, SelectModel,
    SelectTwoModel, SelectorRaw, Set, Statement, StreamTrait, TransactionTrait, Unchanged, Values,
};

pub type DbResult<T> = Result<T, DbErr>;

////////////////////////////////////////////////////////////////////////////////

pub trait ModelRsx<E>
where
    E: EntityTrait,
{
    fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;

    fn merge_from<A>(&mut self, src: A)
    where
        A: ActiveModelTrait<Entity = E>;
}

pub trait ActiveModelRsx<E>
where
    E: EntityTrait,
{
    fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;

    fn merge_from<A>(&mut self, src: A)
    where
        A: ActiveModelTrait<Entity = E>;

    fn set_all(self) -> Self;
}

macro_rules! impl_merge_from {
    ($M:ident, $A:ident) => {
        fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
        where
            S: ?Sized + Contains<C, str>,
            C: Eq + Ord + Hash + Borrow<str>,
        {
            let map = some_or_return!(
                jsn.as_object(),
                Err(DbErr::Type("Invalid JSON object".to_owned()))
            );

            // Mark down which attribute exists in the JSON object
            let json_keys: Vec<<$M::Entity as EntityTrait>::Column> =
                <<$M::Entity as EntityTrait>::Column>::iter()
                    .filter(|col| {
                        let name = col.to_string();
                        !skip.contains_ref(&name) && map.contains_key(&name)
                    })
                    .collect();

            // Convert JSON object into ActiveModel via Model
            let m: <$M::Entity as EntityTrait>::Model =
                serde_json::from_value(jsn).map_err(|e| DbErr::Json(e.to_string()))?;

            for col in json_keys {
                self.set(col, m.get(col));
            }

            Ok(())
        }

        fn merge_from<$A>(&mut self, src: $A)
        where
            $A: ActiveModelTrait<Entity = E>,
        {
            for col in <<$A::Entity as EntityTrait>::Column>::iter() {
                if let ActiveValue::Set(v) = src.get(col) {
                    self.set(col, v);
                }
            }
        }
    };
}

impl<E, M> ModelRsx<E> for M
where
    E: EntityTrait<Model = M>,
    M: ModelTrait<Entity = E> + DeserializeOwned,
{
    impl_merge_from! {M, A}
}

impl<E, A> ActiveModelRsx<E> for A
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E>,
    <E as EntityTrait>::Model: ModelTrait<Entity = E> + DeserializeOwned,
{
    impl_merge_from! {A, A1}

    fn set_all(mut self) -> Self {
        for col in <<A::Entity as EntityTrait>::Column>::iter() {
            if let ActiveValue::Unchanged(v) = self.get(col) {
                self.set(col, v);
            }
        }
        self
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deref, Debug)]
pub struct IdenStr<T: AsRef<str> + Clone + Send + Sync>(pub T);

impl<T: AsRef<str> + Clone + Send + Sync> Iden for IdenStr<T> {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        let _ = s.write_str(self.0.as_ref());
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DbLockMode {
    /// Protects a table from concurrent data changes. (SQLite: BEGIN)
    Share,
    /// Do not allow other data changes. (SQLite: BEGIN IMMEDIATE)
    Exclusive,
    /// Do not allow any other access. (SQLite: BEGIN EXCLUSIVE)
    AccessExclusive,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DbFunc {
    Least,
    Greatest,
    /// Func::current_timestamp()
    Now,
    /// Func::upper()
    Upper,
    /// Func::lower()
    Lower,
}

pub trait DbBackendTrait {
    fn backend(&self) -> DbBackend;

    fn lock_table_sql(&self, table: &str, mode: DbLockMode) -> DbResult<String> {
        _db_lock_table_sql(self.backend(), table, mode)
    }

    fn func_name(&self, func: DbFunc) -> &'static str {
        _db_builtin_func(self.backend(), func)
    }

    fn cust_with_values<S, V, I>(&self, s: S, v: I) -> SimpleExpr
    where
        S: AsRef<str>,
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        Expr::cust_with_values(_db_cust_with_values(self.backend(), s.as_ref()).as_str(), v)
    }

    fn and_optional<P, C>(&self, param: P, condition: C) -> Condition
    where
        P: Into<Value>,
        C: Into<ConditionExpression>,
    {
        Condition::any()
            .add(self.cust_with_values("0 = ?", [P::into(param)]))
            .add(C::into(condition))
    }

    fn or_optional<P, C>(&self, param: P, condition: C) -> Condition
    where
        P: Into<Value>,
        C: Into<ConditionExpression>,
    {
        Condition::all()
            .add(self.cust_with_values("0 <> ?", [P::into(param)]))
            .add(C::into(condition))
    }

    fn now(&self) -> SimpleExpr {
        Func::current_timestamp()
    }

    fn least<T>(&self, arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Func::cust(IdenStr(_db_builtin_func(self.backend(), DbFunc::Least)))
            .args(vec![T::into(arg)])
    }

    fn greatest<T>(&self, arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        Func::cust(IdenStr(_db_builtin_func(self.backend(), DbFunc::Greatest)))
            .args(vec![T::into(arg)])
    }
}

#[async_trait]
pub trait DbConnectionTrait: ConnectionTrait + DbBackendTrait {
    async fn lock_table(&self, table: &str, mode: DbLockMode) -> DbResult<()> {
        let backend = self.backend();
        match backend.lock_table_sql(table, mode) {
            Ok(sql) => {
                if !sql.is_empty() {
                    self.execute(Statement::from_string(backend, sql)).await?;
                }
            }
            _ => return Err(DbErr::Custom("no implementation".to_owned())),
        }
        Ok(())
    }
}

#[async_trait]
impl DbConnectionTrait for DatabaseConnection {}

#[async_trait]
impl DbConnectionTrait for DatabaseTransaction {}

fn _db_builtin_func(backend: DbBackend, func: DbFunc) -> &'static str {
    match backend {
        DbBackend::Postgres => match func {
            DbFunc::Now => "CURRENT_TIMESTAMP",
            DbFunc::Least => "LEAST",
            DbFunc::Greatest => "GREATEST",
            DbFunc::Upper => "UPPER",
            DbFunc::Lower => "LOWER",
        },
        DbBackend::Sqlite => match func {
            DbFunc::Now => "CURRENT_TIMESTAMP",
            DbFunc::Least => "MIN",
            DbFunc::Greatest => "MAX",
            DbFunc::Upper => "UPPER",
            DbFunc::Lower => "LOWER",
        },
        _ => unimplemented!(),
    }
}

fn _db_lock_table_sql(backend: DbBackend, table: &str, mode: DbLockMode) -> DbResult<String> {
    match backend {
        DbBackend::Postgres => {
            if !table.is_empty() {
                let mode = match mode {
                    DbLockMode::Share => "SHARE",
                    DbLockMode::Exclusive => "EXCLUSIVE",
                    DbLockMode::AccessExclusive => "ACCESS EXCLUSIVE",
                };
                Ok(format!("LOCK TABLE {} IN {} MODE;", table, mode))
            } else {
                Ok(String::new())
            }
        }
        DbBackend::Sqlite => Ok(String::new()),
        _ => Err(DbErr::Custom("no implementation".to_owned())),
    }
}

pub fn _db_cust_with_values(backend: DbBackend, s: &str) -> String {
    let mut s = s.as_bytes();
    let mut no = 1;
    let mut buf = Vec::<u8>::with_capacity(s.len() + 32);
    while let Some(i) = s.iter().position(|&x| x == '?' as u8) {
        if s.get(i + 1) == Some(&('?' as u8)) {
            buf.put_slice(&s[..i + 1]);
            s = &s[i + 2..];
        } else {
            buf.put_slice(&s[..i]);
            s = &s[i + 1..];
            match backend {
                DbBackend::Postgres => {
                    write!(&mut buf, "${}", no).unwrap();
                }
                DbBackend::Sqlite | DbBackend::MySql => {
                    buf.put_u8('?' as u8);
                }
            }
            no += 1;
        }
    }
    buf.put_slice(s);
    String::from_utf8(buf).unwrap()
}

impl DbBackendTrait for DbBackend {
    #[inline]
    fn backend(&self) -> DbBackend {
        *self
    }
}

impl DbBackendTrait for DatabaseConnection {
    #[inline]
    fn backend(&self) -> DbBackend {
        self.get_database_backend()
    }
}

impl DbBackendTrait for DatabaseTransaction {
    #[inline]
    fn backend(&self) -> DbBackend {
        self.get_database_backend()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct RawSqlBuilder {
    db_backend: DbBackend,
    builder: Box<dyn QueryBuilder + Send>,
    writer: SqlWriter,
    values: Vec<Value>,
}

impl RawSqlBuilder {
    pub fn new(db_backend: DbBackend) -> Self {
        Self {
            db_backend,
            // The conversion is safe.
            builder: unsafe { mem::transmute(db_backend.get_query_builder()) },
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

    pub fn write_with_args<S, V, I>(&mut self, s: S, v: I)
    where
        S: AsRef<str>,
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        self.write_expr(&self.db_backend.cust_with_values(s, v));
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

#[derive(Clone, Debug)]
pub enum SqlString {
    String(String),
    Shared(ByteString),
}

impl SqlString {
    pub fn into_string(self) -> String {
        match self {
            Self::String(v) => v,
            Self::Shared(v) => v.deref().to_owned(),
        }
    }

    pub fn into_shared(self) -> ByteString {
        match self {
            Self::String(v) => ByteString::from(v),
            Self::Shared(v) => v,
        }
    }
}

impl Default for SqlString {
    fn default() -> Self {
        Self::Shared(ByteString::new())
    }
}

impl Deref for SqlString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::String(v) => v.as_str(),
            Self::Shared(v) => v.deref(),
        }
    }
}

impl AsRef<str> for SqlString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

#[derive(Clone, Debug)]
pub struct SqlHelper {
    sql: SqlString,
    pub values: Option<Values>,
    pub db_backend: DbBackend,

    sql_slices: Vec<ByteString>,
    params: Arc<ParamMap>,
}

impl SqlHelper {
    pub fn into_statement(self) -> Statement {
        let Self {
            sql,
            values,
            db_backend,
            sql_slices,
            ..
        } = self;

        let sql = if sql_slices.is_empty() {
            match sql {
                SqlString::String(v) => v,
                SqlString::Shared(v) => v.deref().to_owned(),
            }
        } else {
            let len = sql_slices.iter().fold(0usize, |n, x| n + x.len());
            let mut sql = String::with_capacity(len);
            sql_slices
                .iter()
                .for_each(|x| sql.write_str(x.deref()).unwrap());
            sql
        };

        Statement {
            sql,
            values,
            db_backend,
        }
    }

    pub fn into_select<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        M::find_by_statement(self.into_statement())
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
    pub fn sql(&self) -> &str {
        self.sql.deref()
    }

    pub fn iter_params(&self) -> SqlParamIterator {
        let params = self.params.clone();
        SqlParamIteratorBuilder {
            params,
            it_builder: |x| x.iter(),
        }
        .build()
    }

    pub fn bind_param<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: AsRef<str>,
        V: Into<Value>,
    {
        let value = value.into();
        for &idx in self.params.get(name.as_ref()).unwrap() {
            match idx {
                ParamIndex::Value(i) => {
                    if let Some(ref mut values) = self.values {
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
        RawSqlBuilder::expr_to_string(self.db_backend, expr)
    }
}

impl From<Statement> for SqlHelper {
    fn from(statement: Statement) -> Self {
        let Statement {
            sql,
            values,
            db_backend,
        } = statement;

        let mut params = ParamMap::new();
        let mut sql_slices = Vec::<ByteString>::new();

        // Get value indices.
        if let Some(values) = &values {
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
        let mut sql_bytes = Bytes::new();
        let mut start = 0;
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"(?:\{\{|\}\}|\{:[[:word:]]+\})"#).unwrap());
        while let Some(m) = RE.find_at(sql.as_str(), start) {
            if sql_bytes.is_empty() {
                sql_bytes = Bytes::copy_from_slice(sql.as_bytes());
            }

            if m.end() - m.start() == 2 {
                // "{{" or "}}"
                sql_slices.push(unsafe {
                    ByteString::from_bytes_unchecked(sql_bytes.slice(start..m.start() + 1))
                });
            } else {
                // Push SQL text before the parameter.
                sql_slices.push(unsafe {
                    ByteString::from_bytes_unchecked(sql_bytes.slice(start..m.start()))
                });

                // Push the parameter: {:<name>}
                sql_slices
                    .push(unsafe { ByteString::from_bytes_unchecked(sql_bytes.slice(m.range())) });

                let name = unsafe {
                    ByteString::from_bytes_unchecked(sql_bytes.slice(m.start() + 1..m.end() - 1))
                };
                params
                    .raw_entry_mut()
                    .from_key(&name)
                    .or_insert_with(move || (name, ParamIndices::new()))
                    .1
                    .push(ParamIndex::Sql((sql_slices.len() - 1) as u32));
            }
            start = m.end();
        }
        let sql = if sql_bytes.is_empty() {
            SqlString::String(sql)
        } else {
            sql_slices.push(unsafe { ByteString::from_bytes_unchecked(sql_bytes.slice(start..)) });
            SqlString::Shared(unsafe { ByteString::from_bytes_unchecked(sql_bytes) })
        };

        //println!("{:?} {:?}", &statement, &params);
        Self {
            sql,
            values,
            db_backend,
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

pub struct SqlCache {
    map: ShardedLock<LinkedHashMap<String, Arc<SqlHelper>>>,
}

impl SqlCache {
    pub fn new() -> Self {
        Self {
            map: ShardedLock::new(LinkedHashMap::new()),
        }
    }

    pub fn get<N, F>(&self, name: N, db_backend: DbBackend, maker: F) -> SqlHelper
    where
        N: AsRef<str>,
        F: FnOnce(DbBackend) -> SqlHelper,
    {
        let name = format!("{:?}://{}", db_backend, name.as_ref());

        // Get from the cache at first.
        let sql = {
            let guard = self.map.read().unwrap();
            match guard.get(&name) {
                Some(v) => v.clone(),
                _ => {
                    drop(guard);
                    // Insert a new SQL.
                    let sql = Arc::new(maker(db_backend));
                    self.map
                        .write()
                        .unwrap()
                        .raw_entry_mut()
                        .from_key(&name)
                        .or_insert(name, sql)
                        .1
                        .clone()
                }
            }
        };
        sql.deref().clone()
    }

    pub fn remove<N>(&self, name: N, db_backend: DbBackend) -> Option<Arc<SqlHelper>>
    where
        N: AsRef<str>,
    {
        let name = format!("{:?}://{}", db_backend, name.as_ref());
        self.map.write().unwrap().remove(&name)
    }

    pub fn clear(&self) {
        self.map.write().unwrap().clear();
    }
}

////////////////////////////////////////////////////////////////////////////////

struct OrderByField {
    field: String,
    asc: bool,
    wrapper_func: IdenStr<ByteString>,
    aggregate_func: IdenStr<ByteString>,
}

pub struct OrderByHelper {
    entity: DynIden,
    id_field: String,
    order_by: Vec<OrderByField>,
}

impl OrderByHelper {
    pub fn new<T>(entity: T) -> Self
    where
        T: IntoIden,
    {
        Self {
            entity: entity.into_iden(),
            id_field: String::new(),
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
        E::Model: DeserializeOwned,
    {
        // filters
        if let Some(after @ &Json::Object(_)) = after {
            if let Ok(model) = serde_json::from_value::<E::Model>(after.clone()) {
                // Filter: "<id_field>" <> after.<id_field>
                let id_col_name = self.id_field.split('.').last().unwrap();
                if let Ok(id_col) = E::Column::from_str(id_col_name) {
                    let after_id = model.get(id_col);
                    if !sea_orm::sea_query::sea_value_to_json_value(&after_id).is_null() {
                        writer(Expr::tbl(self.entity.clone(), id_col).ne(after_id));
                    }
                } else {
                    for key in <E as EntityTrait>::PrimaryKey::iter() {
                        let col = key.into_column();
                        let value = model.get(col);
                        if !sea_orm::sea_query::sea_value_to_json_value(&value).is_null() {
                            writer(Expr::tbl(self.entity.clone(), col).ne(value));
                        }
                    }
                }

                for pat in self.order_by.iter() {
                    if let Ok(col) = E::Column::from_str(&pat.field) {
                        match after.get(&pat.field) {
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
        E::Model: DeserializeOwned,
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
        E::Model: DeserializeOwned,
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
    use crate::db::tests::user::FaRecState;

    mod user {
        use super::*;

        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            AsRefStr,
            EnumIter,
            EnumMessage,
            TryFromPrimitive,
            Serialize_repr,
            Deserialize_repr,
            SmartDefault,
            DeriveActiveEnum,
        )]
        #[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
        #[repr(i16)]
        pub enum FaRecState {
            #[default]
            #[strum(message = "Normal")]
            #[sea_orm(num_value = 1)]
            Normal = 1,
            #[strum(message = "Disabled")]
            #[sea_orm(num_value = 2)]
            Disabled = 2,
            #[strum(message = "Deleted")]
            #[sea_orm(num_value = 3)]
            Deleted = 3,
        }

        #[derive(
            Clone, Debug, Serialize, Deserialize, PartialEq, SmartDefault, DeriveEntityModel,
        )]
        #[sea_orm(table_name = "t_user")]
        pub struct Model {
            #[sea_orm(primary_key)]
            #[serde(default)]
            pub id: i64,
            #[serde(default)]
            pub state: FaRecState,
            #[serde(default)]
            pub role: i16,
            pub name: Option<String>,
            pub nickname: Option<String>,
            pub email: Option<String>,
            pub mobile: Option<String>,
            pub gender: Option<i16>,
            pub birth_year: Option<i32>,
            #[serde(default = "utc_default")]
            #[default(_code = "utc_default()")]
            pub create_time: DateTimeUtc,
            #[serde(default)]
            pub password_hash: String,
            #[serde(default)]
            pub salt: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter)]
        pub enum Relation {}

        impl RelationTrait for Relation {
            fn def(&self) -> RelationDef {
                panic!("No RelationDef")
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_sql_helper() {
        assert_eq!(
            Query::select()
                .expr(DbBackend::Postgres.cust_with_values("? '??' ?", vec!["a", "b"]))
                .to_owned()
                .to_string(sea_orm::sea_query::PostgresQueryBuilder),
            "SELECT 'a' '?' 'b'"
        );
        assert_eq!(
            Query::select()
                .expr(DbBackend::Sqlite.cust_with_values("? '??' ?", vec!["a", "b"]))
                .to_owned()
                .to_string(sea_orm::sea_query::SqliteQueryBuilder),
            "SELECT 'a' '?' 'b'"
        );

        let cache = SqlCache::new();
        for _ in 0..10 {
            let mut q = cache.get("SQL1", DbBackend::Postgres, |be| {
                let mut w = RawSqlBuilder::new(be);
                w.write("SELECT * FROM t_user\n");
                w.write_with_args("WHERE name = ?\n", [":name"]);
                w.write("FOR UPDATE");
                SqlHelper::from(w)
            });
            q.bind_param(":name", "Tom");
            let statement = q.into_statement();
            assert_eq!(
                &statement.sql,
                "SELECT * FROM t_user\nWHERE name = $1\nFOR UPDATE"
            );

            let mut q = cache.get("SQL2", DbBackend::Postgres, |be| {
                let mut w = RawSqlBuilder::new(be);
                w.write("SELECT {{*}} FROM t_user\n");
                w.write_with_args("WHERE name = ? AND nickname = ?\n", [":name", "Mike"]);
                w.write_with_args("AND mobile = ?\n", [":mobile"]);
                w.write("{:order_by}\n");
                w.write("{:limit}\n");
                w.write("{:order_by}\n");
                w.write("FOR UPDATE");
                SqlHelper::from(w)
            });
            q.bind_param(":name", "Tom");
            q.bind_param(":mobile", "123456789");
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
                "SELECT {*} FROM t_user\nWHERE name = $1 AND nickname = $2\nAND mobile = $3\nORDER BY name\nLIMIT 100\nORDER BY name\nFOR UPDATE"
            );
        }
    }

    #[test]
    fn test_active_model() {
        let mut jsn = json!({
            "id": 100,
            "name": "system",
            "xxx": "xxx",
            "state": 8,
            "create_time": "2022-01-01T01:02:03.123456Z",
        });

        jsn.insert_s("state", 8);
        assert!(user::ActiveModel::from_json(jsn.clone()).is_err());
        jsn.insert_s("state", -1);
        assert!(user::ActiveModel::from_json(jsn.clone()).is_err());
        jsn.insert_s("state", FaRecState::Deleted);
        assert!(user::ActiveModel::from_json(jsn.clone()).is_ok());

        let am = user::ActiveModel::from_json(jsn.clone()).unwrap();
        println!("{:?}", &am);

        let user: user::Model = serde_json::from_value(jsn.clone()).unwrap();
        println!("{:?}", &user);

        println!("{:?}", serde_json::to_value(&user));

        let mut am = <user::ActiveModel as Default>::default();
        jsn.insert_s("state", 8);
        assert!(am.merge_from_json(jsn.clone(), &None::<&str>).is_err());
        jsn.insert_s("state", FaRecState::Normal);
        am.merge_from_json(jsn.clone(), &None::<&str>).unwrap();
        println!("{:?}", &am);

        let mut m = user::Model::default();
        jsn.insert_s("state", 8);
        assert!(m.merge_from_json(jsn.clone(), &None::<&str>).is_err());
        jsn.insert_s("state", FaRecState::Normal);
        m.merge_from_json(jsn.clone(), &None::<&str>).unwrap();
        println!("{:?}", &m);
    }
}

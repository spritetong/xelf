//! Database utility components and helper module.
//!
//! This module extends and enhances SeaORM by providing a suite of practical tools,
//! including JSON merging, named parameter SQL construction, SQL template caching,
//! and dynamic cursor-based filtering and pagination/sorting.

#![allow(ambiguous_glob_reexports)]
#![allow(clippy::missing_transmute_annotations)]

use crate::prelude::*;
use derive_more::derive as dm;
use indexmap::IndexMap;
pub use sea_orm::{
    entity::prelude::*,
    sea_query::{
        sea_value_to_json_value, BinOper, DynIden, Expr, ExprTrait, Func, FunctionCall, IntoIden,
        JoinOn, LikeExpr, LogicalChainOper, MysqlQueryBuilder, PostgresQueryBuilder, Query,
        QueryBuilder, SimpleExpr, SqlWriter, SqlWriterValues, SqliteQueryBuilder, UnOper,
    },
    ActiveValue, Condition, ConnectOptions, ConnectionTrait, Database, DatabaseBackend,
    DatabaseTransaction, DbBackend, DbErr, ExecResult, FromQueryResult, IntoActiveModel, JoinType,
    NotSet, Order, QueryOrder, QuerySelect, QueryTrait, SelectGetableValue, SelectModel,
    SelectTwoModel, SelectorRaw, Set, Statement, StreamTrait, TransactionTrait, Unchanged, Values,
};
use std::sync::LazyLock;

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbErr>;

////////////////////////////////////////////////////////////////////////////////

/// Extension trait for SeaORM [ModelTrait], supporting merging updates from a JSON object.
pub trait ModelXlf<E>
where
    E: EntityTrait,
{
    /// Merges properties from a JSON object into the current Model, with an optional skip list.
    fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;

    /// Merges set fields (`ActiveValue::Set`) from a source [ActiveModelTrait] into the current Model.
    fn merge_from<A>(&mut self, src: A)
    where
        A: ActiveModelTrait<Entity = E>;
}

/// Extension trait for SeaORM [ActiveModelTrait], supporting JSON merging and explicit state updates.
pub trait ActiveModelXlf<E>
where
    E: EntityTrait,
{
    /// Merges properties from a JSON object into the current ActiveModel, with an optional skip list.
    fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
    where
        S: ?Sized + Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;

    /// Merges set fields (`ActiveValue::Set`) from a source [ActiveModelTrait] into the current ActiveModel.
    fn merge_from<A>(&mut self, src: A)
    where
        A: ActiveModelTrait<Entity = E>;

    /// Marks all unchanged (`ActiveValue::Unchanged`) fields in the ActiveModel as Set.
    fn set_all(self) -> Self;
}

macro_rules! impl_merge_from {
    ($M:ident, $A:ident) => {
        fn merge_from_json<S, C>(&mut self, jsn: Json, skip: &S) -> DbResult<()>
        where
            S: ?Sized + Contains<C, str>,
            C: Eq + Ord + Hash + Borrow<str>,
        {
            let Some(map) = jsn.as_object() else {
                return Err(DbErr::Type("Invalid JSON object".to_owned()));
            };

            // Mark down which attribute exists in the JSON object
            let json_keys: Vec<<$M::Entity as EntityTrait>::Column> =
                <<$M::Entity as EntityTrait>::Column as sea_orm::Iterable>::iter()
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
            for col in <<$A::Entity as EntityTrait>::Column as sea_orm::Iterable>::iter() {
                if let ActiveValue::Set(v) = src.get(col) {
                    self.set(col, v);
                }
            }
        }
    };
}

impl<E, M> ModelXlf<E> for M
where
    E: EntityTrait<Model = M>,
    M: ModelTrait<Entity = E> + DeserializeOwned,
{
    impl_merge_from! {M, A}
}

impl<E, A> ActiveModelXlf<E> for A
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E>,
    <E as EntityTrait>::Model: ModelTrait<Entity = E> + DeserializeOwned,
{
    impl_merge_from! {A, A1}

    fn set_all(mut self) -> Self {
        for col in <<A::Entity as EntityTrait>::Column as sea_orm::Iterable>::iter() {
            if let ActiveValue::Unchanged(v) = self.get(col) {
                self.set(col, v);
            }
        }
        self
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Wrapper structure for custom database identifiers, allowing raw string passing as SQL identifiers or function names.
#[derive(Clone, dm::Deref, Debug)]
pub struct IdenStr<T: AsRef<str> + Clone + Send + Sync>(pub T);

impl<T: AsRef<str> + Clone + Send + Sync> Iden for IdenStr<T> {
    fn unquoted(&self) -> &str {
        self.0.as_ref()
    }
}

/// Database table lock modes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DbLockMode {
    /// Shared lock / Read lock (Postgres: SHARE, MySQL: READ, SQLite: BEGIN).
    Share,
    /// Exclusive lock / Write lock (Postgres: EXCLUSIVE, MySQL: WRITE, SQLite: BEGIN IMMEDIATE).
    Exclusive,
    /// Access Exclusive lock (Postgres: ACCESS EXCLUSIVE, MySQL: WRITE, SQLite: BEGIN EXCLUSIVE).
    AccessExclusive,
}

/// Generic database built-in functions, automatically mapped across backends (Postgres / MySQL / SQLite).
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DbFunc {
    /// Returns the minimum value (Postgres/MySQL: LEAST, SQLite: MIN).
    Least,
    /// Returns the maximum value (Postgres/MySQL: GREATEST, SQLite: MAX).
    Greatest,
    /// Current timestamp (CURRENT_TIMESTAMP).
    Now,
    /// Converts string to uppercase (UPPER).
    Upper,
    /// Converts string to lowercase (LOWER).
    Lower,
}

/// Abstract trait for database backend behavior, providing cross-backend SQL functions, table lock SQL generation, and conditional helpers.
pub trait DbBackendTrait {
    /// Returns the current database backend ([DbBackend]).
    fn backend(&self) -> DbBackend;

    /// Generates table lock SQL for the current backend.
    fn lock_table_sql(&self, table: &str, mode: DbLockMode) -> DbResult<String> {
        _db_lock_table_sql(self.backend(), table, mode)
    }

    /// Gets the dialect-specific name of a built-in function for the current backend.
    fn func_name(&self, func: DbFunc) -> &'static str {
        _db_builtin_func(self.backend(), func)
    }

    /// Constructs a custom expression with placeholders and values, handling backend placeholder syntax differences automatically.
    fn cust_with_values<S, V, I>(&self, s: S, v: I) -> SimpleExpr
    where
        S: Into<Cow<'static, str>> + AsRef<str>,
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        Expr::cust_with_values(_db_cust_with_values(self.backend(), s).into_owned(), v)
    }

    /// Constructs an optional `AND` filter condition (ignored if parameter value is 0).
    fn and_optional<P, C>(&self, param: P, condition: C) -> Condition
    where
        P: Into<Value>,
        C: Into<Condition>,
    {
        Condition::any()
            .add(self.cust_with_values("0 = ?", [P::into(param)]))
            .add(condition.into())
    }

    /// Constructs an optional `OR` filter condition (evaluates to true if parameter value is non-zero).
    fn or_optional<P, C>(&self, param: P, condition: C) -> Condition
    where
        P: Into<Value>,
        C: Into<Condition>,
    {
        Condition::all()
            .add(self.cust_with_values("0 <> ?", [P::into(param)]))
            .add(condition.into())
    }

    /// Constructs a database function call expression for the current timestamp.
    fn now(&self) -> FunctionCall {
        Func::cust(IdenStr(_db_builtin_func(self.backend(), DbFunc::Now)))
    }

    /// Constructs a database function call expression for minimum value (least/min).
    fn least<T>(&self, arg: T) -> FunctionCall
    where
        T: Into<SimpleExpr>,
    {
        Func::cust(IdenStr(_db_builtin_func(self.backend(), DbFunc::Least))).arg(arg)
    }

    /// Constructs a database function call expression for maximum value (greatest/max).
    fn greatest<T>(&self, arg: T) -> FunctionCall
    where
        T: Into<SimpleExpr>,
    {
        Func::cust(IdenStr(_db_builtin_func(self.backend(), DbFunc::Greatest))).arg(arg)
    }
}

/// Extension trait for database connections and transactions, supporting low-level operations like table locking.
#[async_trait]
pub trait DbConnectionTrait: ConnectionTrait + DbBackendTrait {
    /// Asynchronously executes a table lock operation for the specified table.
    async fn lock_table(&self, table: &str, mode: DbLockMode) -> DbResult<()> {
        let backend = self.backend();
        match backend.lock_table_sql(table, mode) {
            Ok(sql) => {
                if !sql.is_empty() {
                    self.execute_raw(Statement::from_string(backend, sql))
                        .await?;
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
        DbBackend::Postgres | DbBackend::MySql => match func {
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
        _ => match func {
            DbFunc::Now => "CURRENT_TIMESTAMP",
            DbFunc::Least => "LEAST",
            DbFunc::Greatest => "GREATEST",
            DbFunc::Upper => "UPPER",
            DbFunc::Lower => "LOWER",
        },
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
        DbBackend::MySql => {
            if !table.is_empty() {
                let mode = match mode {
                    DbLockMode::Share => "READ",
                    DbLockMode::Exclusive | DbLockMode::AccessExclusive => "WRITE",
                };
                Ok(format!("LOCK TABLES `{}` {};", table, mode))
            } else {
                Ok(String::new())
            }
        }
        DbBackend::Sqlite => Ok(String::new()),
        _ => Err(DbErr::Custom("no implementation".to_owned())),
    }
}

pub fn _db_cust_with_values<T>(backend: DbBackend, s: T) -> Cow<'static, str>
where
    T: Into<Cow<'static, str>> + AsRef<str>,
{
    let mut bytes = s.as_ref().as_bytes();

    if !bytes.contains(&b'?') {
        return s.into();
    }
    if backend != DbBackend::Postgres && !s.as_ref().contains("??") {
        return s.into();
    }

    let mut no = 1;
    let mut buf = Vec::<u8>::with_capacity(bytes.len() + 32);
    while let Some(i) = bytes.iter().position(|&x| x == b'?') {
        if bytes.get(i + 1) == Some(&b'?') {
            buf.put_slice(&bytes[..i + 1]);
            bytes = &bytes[i + 2..];
        } else {
            buf.put_slice(&bytes[..i]);
            bytes = &bytes[i + 1..];
            match backend {
                DbBackend::Postgres => {
                    write!(&mut buf, "${}", no).unwrap();
                }
                _ => {
                    buf.put_u8(b'?');
                }
            }
            no += 1;
        }
    }
    buf.put_slice(bytes);
    Cow::Owned(String::from_utf8(buf).unwrap())
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
        ConnectionTrait::get_database_backend(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Dynamic raw SQL builder that formats expressions and parameters for a target database backend, converting into a [Statement] or [SqlHelper].
pub struct RawSqlBuilder {
    db_backend: DbBackend,
    writer: SqlWriterValues,
}

impl RawSqlBuilder {
    /// Creates a raw SQL builder for the specified database backend.
    pub fn new(db_backend: DbBackend) -> Self {
        let writer = match db_backend {
            DbBackend::MySql | DbBackend::Sqlite => SqlWriterValues::new("?", false),
            DbBackend::Postgres => SqlWriterValues::new("$", true),
            _ => SqlWriterValues::new("?", false),
        };
        Self { db_backend, writer }
    }

    /// Converts the constructed SQL text and parameter collection into a SeaORM [Statement].
    pub fn into_statement(self) -> Statement {
        let (sql, values) = self.writer.into_parts();
        Statement {
            sql,
            values: Some(values),
            db_backend: self.db_backend,
        }
    }

    /// Converts the current builder into a [SqlHelper] supporting named parameter binding.
    pub fn into_sql_helper(self) -> SqlHelper {
        self.into_statement().into()
    }

    /// Converts the current builder into a raw query selector for model `M` ([SelectorRaw<SelectModel<M>>]).
    pub fn into_select<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        SelectorRaw::<SelectModel<M>>::from_statement::<M>(self.into())
    }

    /// Converts the current builder into a raw query selector for a pair of models `(M, N)` ([SelectorRaw<SelectTwoModel<M, N>>]).
    pub fn into_select_two<M, N>(self) -> SelectorRaw<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        // TODO: There's no safe methods to transmute Statement into SelectorRaw<SelectTwoModel>.
        unsafe {
            mem::transmute(SelectorRaw::<SelectModel<M>>::from_statement::<M>(
                self.into(),
            ))
        }
    }

    /// Converts the current builder into a raw query selector returning JSON format.
    pub fn into_json(self) -> SelectorRaw<SelectModel<Json>> {
        SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(self.into())
    }

    /// Converts the current builder into a value query selector ([SelectorRaw<SelectGetableValue<T, C>>]).
    pub fn into_values<T, C>(self) -> SelectorRaw<SelectGetableValue<T, C>>
    where
        T: sea_orm::TryGetableMany,
        C: sea_orm::Iterable + sea_orm::strum::IntoEnumIterator + Iden,
    {
        unsafe {
            mem::transmute(SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(
                self.into(),
            ))
        }
    }

    /// Gets the associated database backend.
    #[inline]
    pub fn get_database_backend(&self) -> DbBackend {
        self.db_backend
    }

    /// Compiles and appends a [SimpleExpr] into the SQL buffer.
    pub fn write_expr(&mut self, expr: &SimpleExpr) {
        match self.db_backend {
            DbBackend::MySql => MysqlQueryBuilder.prepare_expr(expr, &mut self.writer),
            DbBackend::Postgres => PostgresQueryBuilder.prepare_expr(expr, &mut self.writer),
            DbBackend::Sqlite => SqliteQueryBuilder.prepare_expr(expr, &mut self.writer),
            _ => MysqlQueryBuilder.prepare_expr(expr, &mut self.writer),
        }
    }

    /// Appends a raw custom SQL string snippet.
    pub fn write<T>(&mut self, s: T)
    where
        T: Into<Cow<'static, str>>,
    {
        self.write_expr(&Expr::cust(s));
    }

    /// Appends a custom SQL snippet with positional arguments.
    pub fn write_with_args<S, V, I>(&mut self, s: S, v: I)
    where
        S: Into<Cow<'static, str>> + AsRef<str>,
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        self.write_expr(&self.db_backend.cust_with_values(s, v));
    }

    /// Static helper function that compiles a [SimpleExpr] into an SQL string formatted for the target database dialect.
    pub fn expr_to_string(db_backend: DbBackend, expr: &SimpleExpr) -> String {
        let mut w = RawSqlBuilder::new(db_backend);
        w.write_expr(expr);
        w.into_statement().to_string()
    }
}

impl From<RawSqlBuilder> for Statement {
    fn from(builder: RawSqlBuilder) -> Statement {
        builder.into_statement()
    }
}

impl fmt::Debug for RawSqlBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawSqlBuilder")
            .field("db_backend", &self.db_backend)
            .field("SQL", &self.writer)
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
type ParamMap = IndexMap<ByteString, ParamIndices>;

/// SQL template text wrapper enum, supporting owned Strings and shared ByteStrings.
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

/// Advanced SQL helper for named parameters (`:name`) and dynamic SQL slices (`{:slice}`).
#[derive(Clone, Debug)]
pub struct SqlHelper {
    sql: SqlString,
    pub values: Option<Values>,
    pub db_backend: DbBackend,

    sql_slices: Vec<ByteString>,
    params: Arc<ParamMap>,
}

impl SqlHelper {
    /// Renders and exports the parameter-bound [SqlHelper] into an underlying [Statement].
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

    /// Converts into a raw query selector for model `M` ([SelectorRaw<SelectModel<M>>]).
    pub fn into_select<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        SelectorRaw::<SelectModel<M>>::from_statement::<M>(self.into_statement())
    }

    /// Converts into a raw query selector for a pair of models `(M, N)` ([SelectorRaw<SelectTwoModel<M, N>>]).
    pub fn into_select_two<M, N>(self) -> SelectorRaw<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        // TODO: There's no safe methods to transmute Statement into SelectorRaw<SelectTwoModel>.
        unsafe {
            mem::transmute(SelectorRaw::<SelectModel<M>>::from_statement::<M>(
                self.into_statement(),
            ))
        }
    }

    /// Converts into a raw query selector returning JSON format.
    pub fn into_json(self) -> SelectorRaw<SelectModel<Json>> {
        SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(self.into())
    }

    /// Converts into a value query selector ([SelectorRaw<SelectGetableValue<T, C>>]).
    pub fn into_values<T, C>(self) -> SelectorRaw<SelectGetableValue<T, C>>
    where
        T: sea_orm::TryGetableMany,
        C: sea_orm::Iterable + sea_orm::strum::IntoEnumIterator + Iden,
    {
        unsafe {
            mem::transmute(SelectorRaw::<SelectModel<Json>>::from_statement::<Json>(
                self.into(),
            ))
        }
    }

    /// Returns a reference to the current SQL string.
    #[inline]
    pub fn sql(&self) -> &str {
        self.sql.deref()
    }

    /// Returns an iterator over all parameter names parsed from the template.
    pub fn iter_params(&self) -> SqlParamIterator {
        SqlParamIterator {
            params: self.params.clone(),
            index: 0,
        }
    }

    /// Tries to bind a value to the specified parameter name (supporting `:name` parameters and `{:slice}` dynamic SQL blocks), returning an error if missing.
    pub fn try_bind_param<N, V>(&mut self, name: N, value: V) -> DbResult<&mut Self>
    where
        N: AsRef<str>,
        V: Into<Value>,
    {
        let name_str = name.as_ref();
        let Some(indices) = self.params.get(name_str) else {
            return Err(DbErr::Custom(format!(
                "SQL parameter \"{}\" not found in template",
                name_str
            )));
        };

        let value = value.into();
        for &idx in indices {
            match idx {
                ParamIndex::Value(i) => {
                    if let Some(ref mut values) = self.values {
                        values.0[i as usize] = value.clone();
                    }
                }
                ParamIndex::Sql(i) => {
                    if let Value::String(Some(s)) = &value {
                        self.sql_slices[i as usize] = s.deref().into();
                    } else {
                        return Err(DbErr::Custom(format!(
                            "Can not set the SQL slice \"{name_str}\" as {value:?}"
                        )));
                    }
                }
            }
        }
        Ok(self)
    }

    /// Binds a value to the specified parameter name, panicking with a descriptive error message if the parameter name is not found in the template.
    pub fn bind_param<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: AsRef<str>,
        V: Into<Value>,
    {
        let name_str = name.as_ref();
        self.try_bind_param(name_str, value)
            .unwrap_or_else(|e| panic!("{}", e));
        self
    }

    /// Tries to bind an optional condition parameter based on a boolean flag.
    #[inline]
    pub fn try_bind_optional<N: AsRef<str>>(
        &mut self,
        name: N,
        optional: bool,
    ) -> DbResult<&mut Self> {
        self.try_bind_param(name, optional as i32)
    }

    /// Binds an optional condition parameter based on a boolean flag.
    #[inline]
    pub fn bind_optional<N: AsRef<str>>(&mut self, name: N, optional: bool) -> &mut Self {
        self.bind_param(name, optional as i32)
    }

    /// Formats a [SimpleExpr] into an SQL string for the current database dialect.
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
                            .entry(name.deref().into())
                            .or_default()
                            .push(ParamIndex::Value(index as u32));
                    }
                }
            }
        }

        // Get SQL block indices.
        let mut sql_bytes = Bytes::new();
        let mut start = 0;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?:\{\{|\}\}|\{:[[:word:]]+\})").unwrap());
        let re = &*RE;
        while let Some(m) = re.find_at(sql.as_str(), start) {
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
                    .entry(name)
                    .or_default()
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

impl From<SqlHelper> for Statement {
    fn from(helper: SqlHelper) -> Statement {
        helper.into_statement()
    }
}

impl From<RawSqlBuilder> for SqlHelper {
    fn from(builder: RawSqlBuilder) -> Self {
        Statement::from(builder).into()
    }
}

/// Iterator for SQL template parameter names.
pub struct SqlParamIterator {
    params: Arc<ParamMap>,
    index: usize,
}

impl Iterator for SqlParamIterator {
    type Item = ByteString;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, _) = self.params.get_index(self.index)?;
        self.index += 1;
        Some(key.clone())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.params.len().saturating_sub(self.index);
        (len, Some(len))
    }
}

impl ExactSizeIterator for SqlParamIterator {}

////////////////////////////////////////////////////////////////////////////////

/// Thread-safe SQL template cache manager for reusing parsed [SqlHelper] templates.
pub struct SqlCache {
    map: PlRwLock<LinkedHashMap<String, Arc<SqlHelper>>>,
}

impl Default for SqlCache {
    fn default() -> Self {
        Self {
            map: PlRwLock::new(LinkedHashMap::new()),
        }
    }
}

impl SqlCache {
    /// Retrieves a cached [SqlHelper] template clone, calling `maker` to generate and cache it on miss.
    pub fn get<N, F>(&self, name: N, db_backend: DbBackend, maker: F) -> SqlHelper
    where
        N: AsRef<str>,
        F: FnOnce(DbBackend) -> SqlHelper,
    {
        let name = format!("{:?}://{}", db_backend, name.as_ref());

        // Get from the cache at first.
        let sql = {
            let guard = self.map.read();
            match guard.get(&name) {
                Some(v) => v.clone(),
                _ => {
                    drop(guard);
                    // Insert a new SQL.
                    let sql = Arc::new(maker(db_backend));
                    self.map
                        .write()
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

    /// Removes a cached SQL template by name.
    pub fn remove<N>(&self, name: N, db_backend: DbBackend) -> Option<Arc<SqlHelper>>
    where
        N: AsRef<str>,
    {
        let name = format!("{:?}://{}", db_backend, name.as_ref());
        self.map.write().remove(&name)
    }

    /// Clears all cached SQL templates.
    pub fn clear(&self) {
        self.map.write().clear();
    }
}

////////////////////////////////////////////////////////////////////////////////

struct OrderByField {
    field: String,
    asc: bool,
    wrapper_func: IdenStr<ByteString>,
    aggregate_func: IdenStr<ByteString>,
}

/// Helper for dynamic cursor-based filtering and multi-column sorting based on JSON parameters (`after`, `order_by`).
pub struct OrderByHelper {
    entity: DynIden,
    id_field: String,
    order_by: Vec<OrderByField>,
}

impl OrderByHelper {
    /// Constructs a dynamic sorting and filtering helper for the specified entity.
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

    /// Sets the primary key or cursor unique identifier field name.
    pub fn set_id_field<T>(&mut self, id_field: T) -> &mut Self
    where
        T: AsRef<str>,
    {
        self.id_field = id_field.as_ref().to_owned();
        self
    }

    /// Sets ordering rules from a JSON string (e.g. `"id DESC, created_at ASC"`) and optional wrapper/aggregate function mappings.
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
            static RE: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"\b\s*([[:word:]]+)\s*((?i:ASC|DESC)?)\s*\b(?:,|;|$)").unwrap()
            });
            let re = &*RE;
            for cap in re.captures_iter(order_by) {
                self.order_by.push(OrderByField {
                    field: cap[1].to_owned(),
                    asc: !cap[2].eq_ignore_ascii_case("DESC"),
                    wrapper_func: IdenStr(
                        wrapper_funcs
                            .and_then(|x| x.get(&cap[1]).map(|x| x.as_ref().to_owned()))
                            .unwrap_or_default()
                            .into(),
                    ),
                    aggregate_func: IdenStr(
                        aggregate_funcs
                            .and_then(|x| x.get(&cap[1]).map(|x| x.as_ref().to_owned()))
                            .unwrap_or_default()
                            .into(),
                    ),
                });
            }
        }
        self
    }

    /// Applies cursor-based filter conditions (`after`) to a SeaORM [Select] query builder.
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

    /// Applies ordering rules to a SeaORM [Select] query builder.
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

    /// Appends cursor-based filter conditions to a [RawSqlBuilder].
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

    /// Appends ordering rules to a [RawSqlBuilder].
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

    fn write_filters<E>(&self, after: Option<&Json>, writer: &mut dyn FnMut(SimpleExpr))
    where
        E: EntityTrait,
        E::Model: DeserializeOwned,
    {
        // filters
        if let Some(after @ &Json::Object(_)) = after {
            if let Ok(model) = serde_json::from_value::<E::Model>(after.clone()) {
                // Filter: "<id_field>" <> after.<id_field>
                let id_col_name = self.id_field.split('.').next_back().unwrap();
                if let Ok(id_col) = E::Column::from_str(id_col_name) {
                    let after_id = model.get(id_col);
                    if !serde_json::Value::is_null(&sea_value_to_json_value(&after_id)) {
                        writer(Expr::col((self.entity.clone(), id_col)).ne(after_id));
                    }
                } else {
                    for key in <<E as EntityTrait>::PrimaryKey as sea_orm::Iterable>::iter() {
                        let col = key.into_column();
                        let value = model.get(col);
                        if !serde_json::Value::is_null(&sea_value_to_json_value(&value)) {
                            writer(Expr::col((self.entity.clone(), col)).ne(value));
                        }
                    }
                }

                for pat in self.order_by.iter() {
                    if let Ok(col) = E::Column::from_str(&pat.field) {
                        match after.get(&pat.field) {
                            None | Some(Json::Null) => (),
                            _ => {
                                let mut field = Expr::col((self.entity.clone(), col));
                                let mut value = Expr::val(model.get(col));
                                if !pat.wrapper_func.is_empty() {
                                    field =
                                        Expr::expr(Func::cust(pat.wrapper_func.clone()).arg(field));
                                    value =
                                        Expr::expr(Func::cust(pat.wrapper_func.clone()).arg(value))
                                }
                                if pat.asc {
                                    writer(field.gte(value));
                                } else {
                                    writer(field.lte(value));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn write_order_by<E>(&self, writer: &mut dyn FnMut(SimpleExpr, Order))
    where
        E: EntityTrait,
    {
        for pat in self.order_by.iter() {
            if let Ok(col) = E::Column::from_str(&pat.field) {
                let mut field = Expr::col((self.entity.clone(), col));
                if !pat.wrapper_func.is_empty() {
                    field = Func::cust(pat.wrapper_func.clone()).arg(field).into();
                }
                if !pat.aggregate_func.is_empty() {
                    field = Func::cust(pat.aggregate_func.clone()).arg(field).into();
                }
                if pat.asc {
                    writer(field, Order::Asc);
                } else {
                    writer(field, Order::Desc);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::user::RecState;

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
            strum::AsRefStr,
            sea_orm::EnumIter,
            strum::EnumMessage,
            strum::FromRepr,
            Serialize_repr,
            Deserialize_repr,
            SmartDefault,
            DeriveActiveEnum,
        )]
        #[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
        #[repr(i16)]
        pub enum RecState {
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
            pub state: RecState,
            #[serde(default)]
            pub role: i16,
            pub name: Option<String>,
            pub nickname: Option<String>,
            pub email: Option<String>,
            pub mobile: Option<String>,
            pub gender: Option<i16>,
            pub birth_year: Option<i32>,
            #[serde(default = "utc_default")]
            #[default(utc_default())]
            pub create_time: DateTimeUtc,
            #[serde(default)]
            pub password_hash: String,
            #[serde(default)]
            pub salt: String,
        }

        #[derive(Copy, Clone, Debug, sea_orm::EnumIter)]
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
                .expr(DbBackend::Postgres.cust_with_values("? '??' ?", ["a", "b"]))
                .to_owned()
                .to_string(sea_orm::sea_query::PostgresQueryBuilder),
            "SELECT 'a' '?' 'b'"
        );
        assert_eq!(
            Query::select()
                .expr(DbBackend::Sqlite.cust_with_values("? '??' ?", ["a", "b"]))
                .to_owned()
                .to_string(sea_orm::sea_query::SqliteQueryBuilder),
            "SELECT 'a' '?' 'b'"
        );

        let cache = SqlCache::default();
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
        jsn.insert_s("state", RecState::Deleted);
        assert!(user::ActiveModel::from_json(jsn.clone()).is_ok());

        let am = user::ActiveModel::from_json(jsn.clone()).unwrap();
        println!("{:?}", &am);

        let user: user::Model = serde_json::from_value(jsn.clone()).unwrap();
        println!("{:?}", &user);

        println!("{:?}", serde_json::to_value(&user));

        let mut am = <user::ActiveModel as Default>::default();
        jsn.insert_s("state", 8);
        assert!(am.merge_from_json(jsn.clone(), &None::<&str>).is_err());
        jsn.insert_s("state", RecState::Normal);
        am.merge_from_json(jsn.clone(), &None::<&str>).unwrap();
        println!("{:?}", &am);

        let mut m = user::Model::default();
        jsn.insert_s("state", 8);
        assert!(m.merge_from_json(jsn.clone(), &None::<&str>).is_err());
        jsn.insert_s("state", RecState::Normal);
        m.merge_from_json(jsn.clone(), &None::<&str>).unwrap();
        println!("{:?}", &m);
    }
}

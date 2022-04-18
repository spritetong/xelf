#[path = "orm_utils.rs"]
mod orm_utils;

use crate::prelude::*;
use orm_utils::*;

pub use sea_orm::{
    entity::prelude::*,
    sea_query::{ConditionExpression, Expr, Func, Query, SimpleExpr},
    Condition, ConnectionTrait, Database, DatabaseConnection, DatabaseTransaction, DbBackend,
    ExecResult, IntoActiveModel, NotSet, QueryOrder, QuerySelect, QueryTrait, Set, Statement,
    Unchanged,
};

////////////////////////////////////////////////////////////////////////////////

pub trait OrmModelExt {
    fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
    where
        S: Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;
}

pub trait OrmActiveModelExt {
    fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
    where
        S: Contains<C, str>,
        C: Eq + Ord + Hash + Borrow<str>;
}

macro_rules! impl_update_by_json {
    () => {
        fn update_by_json<S, C>(&mut self, jsn: &Json, skip: &S) -> Result<(), DbErr>
        where
            S: Contains<C, str>,
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
    pub statement: Statement,
    params: Arc<ParamMap>,
}

impl SqlHelper {
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
        let Self { statement, .. } = self;
        statement
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

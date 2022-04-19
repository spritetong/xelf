use crate::prelude::*;
use sea_orm::entity::prelude::*;

pub(crate) fn orm_validate_json<T, S, C>(jsn: &JsonMap, skip: &S) -> Result<(), DbErr>
where
    T: Default + Serialize + DeserializeOwned,
    S: ?Sized + Contains<C, str>,
    C: Eq + Ord + Hash + Borrow<str>,
{
    // TODO: no safety ModelTrait::set() ???
    let mut tmp = T::default();
    let mut value = serde_json::to_value(&tmp).map_err(|x| DbErr::Type(x.to_string()))?;
    if let Some(fields) = value.as_object_mut() {
        for (k, v) in fields {
            if !skip.contains_it(&k.as_str()) {
                if let Some(o) = jsn.get(k) {
                    *v = o.clone();
                }
            }
        }
    }
    T::deserialize_in_place(value, &mut tmp).map_err(|x| DbErr::Type(x.to_string()))?;
    Ok(())
}

pub(crate) fn orm_json_to_db_value(ty: &ColumnType, value: &Json) -> Option<Value> {
    type C = ColumnType;
    type V = Value;

    #[inline(always)]
    fn date_time_from<T>(micro_secs: i64, f: fn(i64, u32) -> Option<T>) -> Option<T> {
        f(
            micro_secs.div_euclid(1000000),
            micro_secs.rem_euclid(1000000) as u32 * 1000,
        )
    }

    match value {
        Json::Null => match ty {
            C::Char(_) | C::String(_) | C::Text | C::Custom(_) | C::Enum(_, _) => {
                Some(V::String(None))
            }
            C::TinyInteger => Some(V::TinyInt(None)),
            C::SmallInteger => Some(V::SmallInt(None)),
            C::Integer => Some(V::Int(None)),
            C::BigInteger => Some(V::BigInt(None)),
            C::TinyUnsigned => Some(V::TinyUnsigned(None)),
            C::SmallUnsigned => Some(V::SmallUnsigned(None)),
            C::Unsigned => Some(V::Unsigned(None)),
            C::BigUnsigned => Some(V::BigUnsigned(None)),
            C::Float => Some(V::Float(None)),
            C::Double => Some(V::Double(None)),
            C::Decimal(_) | C::Money(_) => Some(V::Decimal(None)),
            C::DateTime | C::Timestamp => Some(V::ChronoDateTime(None)),
            C::TimestampWithTimeZone => Some(V::ChronoDateTimeUtc(None)),
            C::Time => Some(V::ChronoTime(None)),
            C::Date => Some(V::ChronoDate(None)),
            C::Binary => Some(V::Bytes(None)),
            C::Boolean => Some(V::Bool(None)),
            C::Json | C::JsonBinary => Some(V::Json(None)),
            C::Uuid => Some(V::Uuid(None)),
        },
        Json::Bool(v) => match ty {
            C::Boolean => Some(V::Bool(Some(*v))),
            _ => None,
        },
        Json::Number(v) => match ty {
            C::TinyInteger => match v.as_i64() {
                Some(n) if i8::MIN as i64 <= n && n <= i8::MAX as i64 => {
                    Some(V::TinyInt(Some(n as i8)))
                }
                _ => None,
            },
            C::SmallInteger => match v.as_i64() {
                Some(n) if i16::MIN as i64 <= n && n <= i16::MAX as i64 => {
                    Some(V::SmallInt(Some(n as i16)))
                }
                _ => None,
            },
            C::Integer => match v.as_i64() {
                Some(n) if i32::MIN as i64 <= n && n <= i32::MAX as i64 => {
                    Some(V::Int(Some(n as i32)))
                }
                _ => None,
            },
            C::BigInteger => v.as_i64().map(|x| V::BigInt(Some(x))),

            C::TinyUnsigned => match v.as_u64() {
                Some(n) if u8::MIN as u64 <= n && n <= u8::MAX as u64 => {
                    Some(V::TinyUnsigned(Some(n as u8)))
                }
                _ => None,
            },
            C::SmallUnsigned => match v.as_u64() {
                Some(n) if u16::MIN as u64 <= n && n <= u16::MAX as u64 => {
                    Some(V::SmallUnsigned(Some(n as u16)))
                }
                _ => None,
            },
            C::Unsigned => match v.as_u64() {
                Some(n) if u32::MIN as u64 <= n && n <= u32::MAX as u64 => {
                    Some(V::Unsigned(Some(n as u32)))
                }
                _ => None,
            },
            C::BigUnsigned => v.as_u64().map(|x| V::BigUnsigned(Some(x))),

            C::Float => v.as_f64().map(|x| x as f32).and_then(|x| {
                if !x.is_nan() {
                    Some(V::Float(Some(x)))
                } else {
                    None
                }
            }),
            C::Double => v.as_f64().and_then(|x| {
                if !x.is_nan() {
                    Some(V::Double(Some(x)))
                } else {
                    None
                }
            }),
            C::Decimal(_) | C::Money(_) => v
                .as_f64()
                .and_then(|x| Decimal::from_f64_retain(x))
                .map(|x| V::Decimal(Some(Box::new(x)))),

            C::Boolean => match v.as_i64() {
                Some(n) if n == 1 => Some(V::Bool(Some(true))),
                Some(n) if n == 0 => Some(V::Bool(Some(false))),
                _ => None,
            },

            C::DateTime | C::Timestamp => v
                .as_i64()
                .and_then(|x| date_time_from(x, NaiveDateTime::from_timestamp_opt))
                .map(|x| V::ChronoDateTime(Some(Box::new(x)))),
            C::TimestampWithTimeZone => v
                .as_i64()
                .and_then(|x| {
                    date_time_from(x, |secs, nsecs| Utc.timestamp_opt(secs, nsecs).single())
                })
                .map(|x| V::ChronoDateTimeUtc(Some(Box::new(x)))),
            C::Time => v
                .as_i64()
                .and_then(|x| {
                    date_time_from(x, |secs, nsecs| {
                        Time::from_num_seconds_from_midnight_opt(secs as u32, nsecs)
                    })
                })
                .map(|x| V::ChronoTime(Some(Box::new(x)))),
            C::Date => v
                .as_i64()
                .and_then(|x| NaiveDate::from_num_days_from_ce_opt(x as i32))
                .map(|x| V::ChronoDate(Some(Box::new(x)))),

            C::Enum(_, vars) => v
                .as_i64()
                .and_then(|x| vars.get(x as usize))
                .map(|x| V::String(Some(Box::new(x.clone())))),

            _ => None,
        },
        Json::String(v) => match ty {
            C::Char(_) | C::String(_) | C::Text | C::Custom(_) => {
                Some(V::String(Some(Box::new(v.clone()))))
            }

            C::DateTime | C::Timestamp => NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%.f")
                .ok()
                .map(|x| V::ChronoDateTime(Some(Box::new(x)))),
            C::TimestampWithTimeZone => DateTimeWithTimeZone::parse_from_rfc3339(v)
                .ok()
                .map(|x| V::ChronoDateTimeUtc(Some(Box::new(DateTimeUtc::from(x))))),
            C::Time => Time::parse_from_str(v, "%H:%M:%S%.f")
                .ok()
                .map(|x| V::ChronoTime(Some(Box::new(x)))),
            C::Date => NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .ok()
                .map(|x| V::ChronoDate(Some(Box::new(x)))),

            C::Binary => base64::decode(v).ok().map(|x| V::Bytes(Some(Box::new(x)))),
            C::Json | C::JsonBinary => Json::from_str(v).ok().map(|x| V::Json(Some(Box::new(x)))),
            C::Uuid => Uuid::from_str(v).ok().map(|x| V::Uuid(Some(Box::new(x)))),
            C::Enum(_, vars) => {
                if vars.contains(v) {
                    Some(V::String(Some(Box::new(v.clone()))))
                } else {
                    None
                }
            }
            _ => None,
        },
        Json::Array(_) => match ty {
            C::Json | C::JsonBinary => Some(V::Json(Some(Box::new(value.clone())))),
            _ => None,
        },
        Json::Object(_) => match ty {
            C::Json | C::JsonBinary => Some(V::Json(Some(Box::new(value.clone())))),
            _ => None,
        },
    }
}

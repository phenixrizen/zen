use crate::functions::defs::{
    CompositeFunction, FunctionDefinition, FunctionSignature, StaticFunction,
};
use std::rc::Rc;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Debug, PartialEq, Eq, Hash, Display, EnumString, EnumIter, IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "camelCase")]
pub enum InternalFunction {
    // General
    Len,
    Contains,
    Flatten,
    Merge,
    MergeDeep,

    // String
    Upper,
    Lower,
    Trim,
    StartsWith,
    EndsWith,
    Matches,
    Extract,
    FuzzyMatch,
    Split,

    // Math
    Abs,
    Sum,
    Avg,
    Min,
    Max,
    Rand,
    Median,
    Mode,
    Floor,
    Ceil,
    Round,
    Trunc,

    // Type
    IsNumeric,
    String,
    Number,
    Bool,
    Type,

    // Map
    Keys,
    Values,

    #[strum(serialize = "d")]
    Date,
}

impl From<&InternalFunction> for Rc<dyn FunctionDefinition> {
    fn from(value: &InternalFunction) -> Self {
        use crate::variable::VariableType as VT;
        use InternalFunction as IF;

        let s: Rc<dyn FunctionDefinition> = match value {
            IF::Len => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::len),
                signatures: vec![
                    FunctionSignature::single(VT::String, VT::Number),
                    FunctionSignature::single(VT::Any.array(), VT::Number),
                ],
            }),

            IF::Contains => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::contains),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::String, VT::String],
                        return_type: VT::Bool,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any.array(), VT::Any],
                        return_type: VT::Bool,
                    },
                ],
            }),

            IF::Flatten => Rc::new(StaticFunction {
                implementation: Rc::new(imp::flatten),
                signature: FunctionSignature::single(VT::Any.array(), VT::Any.array()),
            }),

            IF::Merge => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::merge),
                signatures: vec![
                    FunctionSignature::single(VT::Any.array(), VT::Any.array()),
                    FunctionSignature::single(
                        VT::Object(Default::default()).array(),
                        VT::Object(Default::default()),
                    ),
                ],
            }),

            IF::MergeDeep => Rc::new(StaticFunction {
                implementation: Rc::new(imp::merge_deep),
                signature: FunctionSignature::single(
                    VT::Object(Default::default()).array(),
                    VT::Object(Default::default()),
                ),
            }),

            IF::Upper => Rc::new(StaticFunction {
                implementation: Rc::new(imp::upper),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            IF::Lower => Rc::new(StaticFunction {
                implementation: Rc::new(imp::lower),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            IF::Trim => Rc::new(StaticFunction {
                implementation: Rc::new(imp::trim),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            IF::StartsWith => Rc::new(StaticFunction {
                implementation: Rc::new(imp::starts_with),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            IF::EndsWith => Rc::new(StaticFunction {
                implementation: Rc::new(imp::ends_with),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            IF::Matches => Rc::new(StaticFunction {
                implementation: Rc::new(imp::matches),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            IF::Extract => Rc::new(StaticFunction {
                implementation: Rc::new(imp::extract),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::String.array(),
                },
            }),

            IF::Split => Rc::new(StaticFunction {
                implementation: Rc::new(imp::split),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::String.array(),
                },
            }),

            IF::FuzzyMatch => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::fuzzy_match),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::String, VT::String],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::String.array(), VT::String],
                        return_type: VT::Number.array(),
                    },
                ],
            }),

            IF::Abs => Rc::new(StaticFunction {
                implementation: Rc::new(imp::abs),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            IF::Rand => Rc::new(StaticFunction {
                implementation: Rc::new(imp::rand),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            IF::Floor => Rc::new(StaticFunction {
                implementation: Rc::new(imp::floor),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            IF::Ceil => Rc::new(StaticFunction {
                implementation: Rc::new(imp::ceil),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            IF::Round => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::round),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Number],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Number, VT::Number],
                        return_type: VT::Number,
                    },
                ],
            }),

            IF::Trunc => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::trunc),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Number],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Number, VT::Number],
                        return_type: VT::Number,
                    },
                ],
            }),

            IF::Sum => Rc::new(StaticFunction {
                implementation: Rc::new(imp::sum),
                signature: FunctionSignature::single(VT::Number.array(), VT::Number),
            }),

            IF::Avg => Rc::new(StaticFunction {
                implementation: Rc::new(imp::avg),
                signature: FunctionSignature::single(VT::Number.array(), VT::Number),
            }),

            IF::Min => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::min),
                signatures: vec![
                    FunctionSignature::single(VT::Number.array(), VT::Number),
                    FunctionSignature::single(VT::Date.array(), VT::Date),
                ],
            }),

            IF::Max => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::max),
                signatures: vec![
                    FunctionSignature::single(VT::Number.array(), VT::Number),
                    FunctionSignature::single(VT::Date.array(), VT::Date),
                ],
            }),

            IF::Median => Rc::new(StaticFunction {
                implementation: Rc::new(imp::median),
                signature: FunctionSignature::single(VT::Number.array(), VT::Number),
            }),

            IF::Mode => Rc::new(StaticFunction {
                implementation: Rc::new(imp::mode),
                signature: FunctionSignature::single(VT::Number.array(), VT::Number),
            }),

            IF::Type => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_type),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            IF::String => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_string),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            IF::Bool => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_bool),
                signature: FunctionSignature::single(VT::Any, VT::Bool),
            }),

            IF::IsNumeric => Rc::new(StaticFunction {
                implementation: Rc::new(imp::is_numeric),
                signature: FunctionSignature::single(VT::Any, VT::Bool),
            }),

            IF::Number => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_number),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            IF::Keys => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::keys),
                signatures: vec![
                    FunctionSignature::single(VT::Object(Default::default()), VT::String.array()),
                    FunctionSignature::single(VT::Any.array(), VT::Number.array()),
                ],
            }),

            IF::Values => Rc::new(StaticFunction {
                implementation: Rc::new(imp::values),
                signature: FunctionSignature::single(
                    VT::Object(Default::default()),
                    VT::Any.array(),
                ),
            }),

            IF::Date => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::date),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![],
                        return_type: VT::Date,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any],
                        return_type: VT::Date,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any, VT::String],
                        return_type: VT::Date,
                    },
                ],
            }),
        };

        s
    }
}

pub(crate) mod imp {
    use crate::functions::arguments::Arguments;
    use crate::vm::date::DynamicVariableExt;
    use crate::vm::VmDate;
    use crate::{Variable as V, Variable};
    use anyhow::{anyhow, Context};
    use chrono_tz::Tz;
    use hashlink::LruCache;
    #[cfg(not(feature = "regex-lite"))]
    use regex::Regex;
    #[cfg(feature = "regex-lite")]
    use regex_lite::Regex;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::{Decimal, RoundingStrategy};
    use rust_decimal_macros::dec;
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::rc::Rc;
    use std::str::FromStr;

    /// How many compiled patterns each thread keeps. Past it, the least recently used is dropped.
    pub(crate) const REGEX_CACHE_CAPACITY: usize = 32;

    /// Compiled patterns, with least-recently-used eviction. One per thread: a lookup takes no
    /// lock, and a thread's patterns are freed when it exits.
    pub(crate) struct RegexCache(RefCell<LruCache<Box<str>, Rc<Regex>>>);

    impl RegexCache {
        pub(crate) fn new(capacity: usize) -> Self {
            Self(RefCell::new(LruCache::new(capacity)))
        }

        /// The compiled form of `pattern`, compiled on its first use and kept after. An invalid
        /// pattern is not kept, so it fails on every call.
        pub(crate) fn get(&self, pattern: &str) -> anyhow::Result<Rc<Regex>> {
            if let Some(regex) = self.0.borrow_mut().get(pattern) {
                return Ok(regex.clone());
            }

            let regex = Rc::new(Regex::new(pattern).context("Invalid regular expression")?);
            self.0.borrow_mut().insert(pattern.into(), regex.clone());

            Ok(regex)
        }
    }

    thread_local!(static REGEX_CACHE: RegexCache = RegexCache::new(REGEX_CACHE_CAPACITY));

    /// The compiled form of `pattern`, from this thread's cache.
    ///
    /// `matches` and `extract` call this instead of compiling their pattern on every call. The
    /// result is the same, since a pattern always compiles to the same regex; only when it is
    /// compiled changes.
    pub(crate) fn cached_regex(pattern: &str) -> anyhow::Result<Rc<Regex>> {
        REGEX_CACHE.with(|cache| cache.get(pattern))
    }

    fn __internal_number_array(args: &Arguments, pos: usize) -> anyhow::Result<Vec<Decimal>> {
        let a = args.array(pos)?;
        let arr = a.borrow();

        arr.iter()
            .map(|v| v.as_number())
            .collect::<Option<Vec<_>>>()
            .context("Expected a number array")
    }

    enum Either<A, B> {
        Left(A),
        Right(B),
    }

    fn __internal_number_or_date_array(
        args: &Arguments,
        pos: usize,
    ) -> anyhow::Result<Either<Vec<Decimal>, Vec<VmDate>>> {
        let a = args.array(pos)?;
        let arr = a.borrow();

        let is_number = arr.first().map(|v| v.as_number()).flatten().is_some();
        if is_number {
            Ok(Either::Left(
                arr.iter()
                    .map(|v| v.as_number())
                    .collect::<Option<Vec<_>>>()
                    .context("Expected a number array")?,
            ))
        } else {
            Ok(Either::Right(
                arr.iter()
                    .map(|v| match v {
                        Variable::Dynamic(d) => d.as_date().cloned(),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()
                    .context("Expected a number array")?,
            ))
        }
    }

    pub fn starts_with(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        Ok(V::Bool(a.starts_with(b)))
    }

    pub fn ends_with(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        Ok(V::Bool(a.ends_with(b)))
    }

    pub fn matches(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let regex = cached_regex(b.as_ref())?;

        Ok(V::Bool(regex.is_match(a.as_ref())))
    }

    pub fn upper(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.to_uppercase().into()))
    }

    pub fn lower(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.to_lowercase().into()))
    }

    pub fn trim(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.trim().into()))
    }

    pub fn extract(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let regex = cached_regex(b.as_ref())?;

        let captures = regex
            .captures(a.as_ref())
            .map(|capture| {
                capture
                    .iter()
                    .map(|c| c.map(|c| c.as_str()))
                    .filter_map(|c| c)
                    .map(|s| V::String((s).into()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(V::from_array(captures))
    }

    pub fn split(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let arr = Vec::from_iter(
            a.split(b)
                .into_iter()
                .map(|s| V::String(s.to_string().into())),
        );

        Ok(V::from_array(arr))
    }

    pub fn flatten(args: Arguments) -> anyhow::Result<V> {
        let a = args.array(0)?;

        let arr = a.borrow();
        let mut flat_arr = Vec::with_capacity(arr.len());
        arr.iter().for_each(|v| match v {
            V::Array(b) => {
                let arr = b.borrow();
                arr.iter().for_each(|v| flat_arr.push(v.clone()))
            }
            _ => flat_arr.push(v.clone()),
        });

        Ok(V::from_array(flat_arr))
    }

    pub fn merge(args: Arguments) -> anyhow::Result<V> {
        let a = args.array(0)?;
        let arr = a.borrow();

        let Some(first) = arr.iter().find(|item| !matches!(item, V::Null)) else {
            return Ok(V::empty_object());
        };

        let capacity = arr
            .iter()
            .map(|item| match item {
                V::Object(obj) => obj.borrow().len(),
                V::Array(arr) => arr.borrow().len(),
                _ => 0,
            })
            .sum();

        match first {
            V::Array(_) => {
                let mut merged = Vec::with_capacity(capacity);

                for item in arr.iter() {
                    match item {
                        V::Array(inner) => {
                            let inner = inner.borrow();
                            merged.extend(inner.iter().cloned());
                        }
                        V::Null => {}
                        _ => return Err(anyhow!("Expected array of arrays")),
                    }
                }

                Ok(V::from_array(merged))
            }
            V::Object(_) => {
                let mut merged = zen_types::variable::VariableMap::with_capacity(capacity);
                for item in arr.iter() {
                    match item {
                        V::Object(obj) => {
                            let obj = obj.borrow();
                            for (key, value) in obj.iter() {
                                merged.insert(key.clone(), value.clone());
                            }
                        }
                        V::Null => {}
                        _ => return Err(anyhow!("Expected array of objects")),
                    }
                }

                Ok(V::from_object(merged))
            }
            other => Err(anyhow!(
                "merge expects an array of arrays or objects, got {}",
                other.type_name()
            )),
        }
    }

    pub fn merge_deep(args: Arguments) -> anyhow::Result<V> {
        let a = args.array(0)?;
        let arr = a.borrow();

        let mut result = V::empty_object();
        for item in arr.iter() {
            match item {
                V::Object(_) => {
                    result = deep_merge_variables(&result, item);
                }
                V::Null => {}
                _ => return Err(anyhow!("Expected array of objects")),
            }
        }

        Ok(result)
    }

    fn deep_merge_variables(base: &V, patch: &V) -> V {
        match (base, patch) {
            (V::Object(a), V::Object(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                let mut merged = zen_types::variable::VariableMap::with_capacity(a.len() + b.len());

                for (key, value) in a.iter() {
                    merged.insert(key.clone(), value.clone());
                }

                for (key, value) in b.iter() {
                    let entry = merged
                        .get(key)
                        .map(|existing| deep_merge_variables(existing, value))
                        .unwrap_or_else(|| value.clone());
                    merged.insert(key.clone(), entry);
                }

                V::from_object(merged)
            }
            (V::Array(a), V::Array(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                let mut merged = Vec::with_capacity(a.len() + b.len());
                merged.extend(a.iter().cloned());
                merged.extend(b.iter().cloned());
                V::from_array(merged)
            }
            (_, patch) => patch.clone(),
        }
    }

    pub fn abs(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.abs()))
    }

    pub fn ceil(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.ceil()))
    }

    pub fn floor(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.floor()))
    }

    pub fn round(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let dp = args
            .onumber(1)?
            .map(|v| v.to_u32().context("Invalid number of decimal places"))
            .transpose()?
            .unwrap_or(0);

        Ok(V::Number(a.round_dp_with_strategy(
            dp,
            RoundingStrategy::MidpointAwayFromZero,
        )))
    }

    pub fn trunc(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let dp = args
            .onumber(1)?
            .map(|v| v.to_u32().context("Invalid number of decimal places"))
            .transpose()?
            .unwrap_or(0);

        Ok(V::Number(a.trunc_with_scale(dp)))
    }

    pub fn rand(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let upper_range = a.round().to_i64().context("Invalid upper range")?;

        let random_number = fastrand::i64(0..=upper_range);
        Ok(V::Number(Decimal::from(random_number)))
    }

    pub fn min(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_or_date_array(&args, 0)?;

        match a {
            Either::Left(arr) => {
                let max = arr.into_iter().min().context("Empty array")?;
                Ok(V::Number(Decimal::from(max)))
            }
            Either::Right(arr) => {
                let max = arr.into_iter().min().context("Empty array")?;
                Ok(V::Dynamic(Rc::new(max)))
            }
        }
    }

    pub fn max(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_or_date_array(&args, 0)?;

        match a {
            Either::Left(arr) => {
                let max = arr.into_iter().max().context("Empty array")?;
                Ok(V::Number(Decimal::from(max)))
            }
            Either::Right(arr) => {
                let max = arr.into_iter().max().context("Empty array")?;
                Ok(V::Dynamic(Rc::new(max)))
            }
        }
    }

    pub fn avg(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let sum = a
            .iter()
            .try_fold(Decimal::ZERO, |acc, x| acc.checked_add(*x))
            .context("Number overflow")?;

        Ok(V::Number(Decimal::from(
            sum.checked_div(Decimal::from(a.len()))
                .context("Empty array")?,
        )))
    }

    pub fn sum(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let sum = a
            .iter()
            .try_fold(Decimal::ZERO, |acc, v| acc.checked_add(*v))
            .context("Number overflow")?;

        Ok(V::Number(Decimal::from(sum)))
    }

    pub fn median(args: Arguments) -> anyhow::Result<V> {
        let mut a = __internal_number_array(&args, 0)?;
        a.sort();

        let center = a.len() / 2;
        if a.len() % 2 == 1 {
            let center_num = a.get(center).context("Index out of bounds")?;
            Ok(V::Number(*center_num))
        } else {
            let center_left = a.get(center - 1).context("Index out of bounds")?;
            let center_right = a.get(center).context("Index out of bounds")?;

            let median = center_left
                .checked_add(*center_right)
                .context("Number overflow")?
                / dec!(2);
            Ok(V::Number(median))
        }
    }

    pub fn mode(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let mut counts = BTreeMap::new();
        for num in a {
            *counts.entry(num).or_insert(0) += 1;
        }

        let most_common = counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(num, _)| num)
            .context("Empty array")?;

        Ok(V::Number(most_common))
    }

    pub fn to_type(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        Ok(V::String(a.type_name().into()))
    }

    pub fn to_bool(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Null => false,
            V::Bool(v) => *v,
            V::Number(n) => !n.is_zero(),
            V::Array(_) | V::Object(_) | V::Dynamic(_) => true,
            V::String(s) => match s.as_str().trim() {
                "true" => true,
                "false" => false,
                _ => s.is_empty(),
            },
        };

        Ok(V::Bool(val))
    }

    pub fn to_string(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Null => "null".into(),
            V::Bool(v) => v.to_string().into(),
            V::Number(n) => n.to_string().into(),
            V::String(s) => s.clone(),
            _ => return Err(anyhow!("Cannot convert type {} to string", a.type_name())),
        };

        Ok(V::String(val))
    }

    pub fn to_number(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Number(n) => *n,
            V::String(str) => {
                let s = str.as_str().trim();
                Decimal::from_str_exact(s)
                    .or_else(|_| Decimal::from_scientific(s))
                    .context("Invalid number")?
            }
            V::Bool(b) => match *b {
                true => Decimal::ONE,
                false => Decimal::ZERO,
            },
            _ => return Err(anyhow!("Cannot convert type {} to number", a.type_name())),
        };

        Ok(V::Number(val))
    }

    pub fn is_numeric(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let is_ok = match a {
            V::Number(_) => true,
            V::String(str) => {
                let s = str.as_str().trim();
                Decimal::from_str_exact(s)
                    .or_else(|_| Decimal::from_scientific(s))
                    .is_ok()
            }
            _ => false,
        };

        Ok(V::Bool(is_ok))
    }

    pub fn len(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let len = match a {
            V::String(s) => s.len(),
            V::Array(s) => {
                let arr = s.borrow();
                arr.len()
            }
            _ => {
                return Err(anyhow!("Cannot determine len of type {}", a.type_name()));
            }
        };

        Ok(V::Number(len.into()))
    }

    pub fn contains(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let b = args.var(1)?;

        let val = match (a, b) {
            (V::String(a), V::String(b)) => a.contains(b.as_str()),
            (V::Array(a), _) => {
                let arr = a.borrow();

                arr.iter().any(|a| match (a, b) {
                    (V::Number(a), V::Number(b)) => a == b,
                    (V::String(a), V::String(b)) => a == b,
                    (V::Bool(a), V::Bool(b)) => a == b,
                    (V::Null, V::Null) => true,
                    _ => false,
                })
            }
            _ => {
                return Err(anyhow!(
                    "Cannot determine contains for type {} and {}",
                    a.type_name(),
                    b.type_name()
                ));
            }
        };

        Ok(V::Bool(val))
    }

    pub fn fuzzy_match(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let b = args.str(1)?;

        let val = match a {
            V::String(a) => {
                let sim = strsim::normalized_damerau_levenshtein(a.as_ref(), b.as_ref());
                // This is okay, as NDL will return [0, 1]
                V::Number(Decimal::from_f64(sim).unwrap_or(dec!(0)))
            }
            V::Array(_a) => {
                let a = _a.borrow();
                let mut sims = Vec::with_capacity(a.len());
                for v in a.iter() {
                    let s = v.as_str().context("Expected string array")?;

                    let sim = Decimal::from_f64(strsim::normalized_damerau_levenshtein(
                        s.as_ref(),
                        b.as_ref(),
                    ))
                    .unwrap_or(dec!(0));
                    sims.push(V::Number(sim));
                }

                V::from_array(sims)
            }
            _ => return Err(anyhow!("Fuzzy match not available for type")),
        };

        Ok(val)
    }

    pub fn keys(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let var = match a {
            V::Array(a) => {
                let arr = a.borrow();
                let indices = arr
                    .iter()
                    .enumerate()
                    .map(|(index, _)| V::Number(index.into()))
                    .collect();

                V::from_array(indices)
            }
            V::Object(a) => {
                let obj = a.borrow();
                let keys = obj
                    .iter()
                    .map(|(key, _)| V::String((key.as_str()).into()))
                    .collect();

                V::from_array(keys)
            }
            _ => {
                return Err(anyhow!("Cannot determine keys of type {}", a.type_name()));
            }
        };

        Ok(var)
    }

    pub fn values(args: Arguments) -> anyhow::Result<V> {
        let a = args.object(0)?;
        let obj = a.borrow();
        let values: Vec<_> = obj.values().cloned().collect();

        Ok(V::from_array(values))
    }

    pub fn date(args: Arguments) -> anyhow::Result<V> {
        let provided = args.ovar(0);
        let tz = args
            .ostr(1)?
            .map(|v| Tz::from_str(v).context("Invalid timezone"))
            .transpose()?;

        let date_time = match provided {
            Some(v) => VmDate::new(v.clone(), tz),
            None => VmDate::now(),
        };

        Ok(V::Dynamic(Rc::new(date_time)))
    }
}

#[cfg(test)]
mod regex_cache_tests {
    use super::imp::{cached_regex, RegexCache, REGEX_CACHE_CAPACITY};
    use crate::{Isolate, Variable};
    #[cfg(not(feature = "regex-lite"))]
    use regex::Regex;
    #[cfg(feature = "regex-lite")]
    use regex_lite::Regex;
    use serde_json::{json, Value};
    use std::rc::Rc;
    use std::thread;

    /// Patterns and inputs covering anchors, capture groups, optional groups that do not take
    /// part, case-insensitive and Unicode classes, and empty matches.
    const CASES: &[(&str, &str)] = &[
        ("^[0-9]+$", "99213"),
        ("^[0-9]+$", "A0425"),
        ("^[0-9]+$", ""),
        ("^0*(42.)$", "0420"),
        ("^0*(.*)$", "000"),
        ("^(.*?)s*$", "months"),
        ("^([A-Z]*)([0-9]+)([A-Z]*)$", "A0425"),
        ("^([A-Z]*)([0-9]+)([A-Z]*)$", "A-1"),
        ("^(A)?(B)$", "B"),
        ("(?i)^[ s]*day[ s]*$", " DAYS "),
        ("^\\p{L}", "é1234"),
        ("(?s)^.{4}[MU]", "abc\nM"),
        ("(\\d{4})-(\\d{2})-(\\d{2})", "on 2022-09-18"),
        ("", "anything"),
    ];

    /// The expression's answer, or `None` where it fails.
    fn run(expression: &str, input: &str, pattern: &str) -> Option<Value> {
        let env = Variable::from(json!({ "input": input, "pattern": pattern }));
        Isolate::with_environment(env)
            .run_standard(expression)
            .ok()
            .map(|v| v.to_value())
    }

    /// What `matches` and `extract` gave before the cache: a pattern compiled for the call, or
    /// `None` for both where the pattern does not compile (`regex-lite` has no `\p{L}`).
    fn uncached(pattern: &str, input: &str) -> (Option<Value>, Option<Value>) {
        let Ok(regex) = Regex::new(pattern) else {
            return (None, None);
        };
        let captures: Vec<&str> = regex
            .captures(input)
            .map(|c| c.iter().flatten().map(|m| m.as_str()).collect())
            .unwrap_or_default();
        (Some(json!(regex.is_match(input))), Some(json!(captures)))
    }

    #[test]
    fn matches_and_extract_answer_as_without_the_cache() {
        for (pattern, input) in CASES {
            let (is_match, captures) = uncached(pattern, input);
            // The first call compiles the pattern, the second finds it in the cache.
            for _ in 0..2 {
                assert_eq!(
                    run("matches(input, pattern)", input, pattern),
                    is_match,
                    "matches({input:?}, {pattern:?})"
                );
                assert_eq!(
                    run("extract(input, pattern)", input, pattern),
                    captures,
                    "extract({input:?}, {pattern:?})"
                );
            }
        }
    }

    #[test]
    fn an_invalid_pattern_fails_on_every_call() {
        for _ in 0..3 {
            let error = cached_regex("([0-9]").unwrap_err();
            assert_eq!(error.to_string(), "Invalid regular expression");
            for expression in ["matches(\"1\", \"([0-9]\")", "extract(\"1\", \"([0-9]\")"] {
                assert!(
                    Isolate::new().run_standard(expression).is_err(),
                    "{expression} did not fail"
                );
            }
        }
    }

    #[test]
    fn a_pattern_is_compiled_once() {
        let cache = RegexCache::new(4);
        let first = cache.get("^once-[0-9]+$").unwrap();
        let second = cache.get("^once-[0-9]+$").unwrap();
        assert!(Rc::ptr_eq(&first, &second));
    }

    #[test]
    fn a_recently_used_pattern_survives_eviction_and_the_least_recently_used_does_not() {
        let cache = RegexCache::new(2);
        let a = cache.get("^a$").unwrap();
        let b = cache.get("^b$").unwrap();
        // Using `a` makes `b` the least recently used, so `c` evicts `b`.
        assert!(Rc::ptr_eq(&a, &cache.get("^a$").unwrap()));
        cache.get("^c$").unwrap();

        assert!(Rc::ptr_eq(&a, &cache.get("^a$").unwrap()), "a was evicted");
        let b_again = cache.get("^b$").unwrap();
        assert!(!Rc::ptr_eq(&b, &b_again), "b was not evicted");
        assert!(b_again.is_match("b"));
    }

    #[test]
    fn a_thread_keeps_at_most_the_capacity() {
        // Each test runs on a thread of its own, so this thread's cache starts empty.
        let first = cached_regex("^capacity-0$").unwrap();
        for i in 1..REGEX_CACHE_CAPACITY {
            cached_regex(&format!("^capacity-{i}$")).unwrap();
        }
        // The cache is full, and `first` is still in it.
        assert!(Rc::ptr_eq(&first, &cached_regex("^capacity-0$").unwrap()));

        // One more pattern evicts the least recently used, `^capacity-1$`, not `first`.
        let second = cached_regex("^capacity-1$").unwrap();
        cached_regex("^capacity-extra$").unwrap();
        assert!(Rc::ptr_eq(&first, &cached_regex("^capacity-0$").unwrap()));
        assert!(Rc::ptr_eq(&second, &cached_regex("^capacity-1$").unwrap()));
    }

    #[test]
    fn each_thread_compiles_its_own() {
        let address = |regex: Rc<Regex>| Rc::as_ptr(&regex) as usize;
        let here = address(cached_regex("^per-thread$").unwrap());
        let there = thread::spawn(move || address(cached_regex("^per-thread$").unwrap()))
            .join()
            .unwrap();
        assert_ne!(here, there);
    }
}

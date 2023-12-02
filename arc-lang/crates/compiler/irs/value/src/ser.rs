use crate::*;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.kind.as_ref() {
            VAggregator(v) => unreachable!(),
            VArray(v) => v.0.serialize(serializer),
            VBlob(v) => v.serialize(serializer),
            VBool(v) => v.serialize(serializer),
            VChar(v) => v.serialize(serializer),
            VDict(v) => v.serialize(serializer),
            VDiscretizer(v) => v.serialize(serializer),
            VDuration(v) => v.serialize(serializer),
            VEncoding(v) => v.serialize(serializer),
            VF32(v) => v.serialize(serializer),
            VF64(v) => v.serialize(serializer),
            VFile(v) => unreachable!(),
            VFunction(v) => unreachable!(),
            VI128(v) => v.serialize(serializer),
            VI16(v) => v.serialize(serializer),
            VI32(v) => v.serialize(serializer),
            VI64(v) => v.serialize(serializer),
            VI8(v) => v.serialize(serializer),
            VMatrix(v) => v.serialize(serializer),
            VModel(v) => v.serialize(serializer),
            VOption(v) => v.serialize(serializer),
            VPath(v) => v.serialize(serializer),
            VReader(v) => v.serialize(serializer),
            VRecord(v) => v.serialize(serializer),
            VResult(v) => v.serialize(serializer),
            VSet(v) => v.serialize(serializer),
            VSocketAddr(v) => v.serialize(serializer),
            VStream(v) => unreachable!(),
            VString(v) => v.serialize(serializer),
            VTime(v) => v.serialize(serializer),
            VTimeSource(v) => unreachable!(),
            VTuple(v) => v.serialize(serializer),
            VU128(v) => v.serialize(serializer),
            VU16(v) => v.serialize(serializer),
            VU32(v) => v.serialize(serializer),
            VU64(v) => v.serialize(serializer),
            VU8(v) => v.serialize(serializer),
            VUnit(v) => v.serialize(serializer),
            VUrl(v) => v.serialize(serializer),
            VUsize(v) => v.serialize(serializer),
            VVariant(v) => v.serialize(serializer),
            VVec(v) => v.serialize(serializer),
            VWriter(v) => v.serialize(serializer),
            VDataflow(v) => unreachable!(),
            VInstance(v) => unreachable!(),
            VImage(v) => v.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod test {
    use im_rc::hashmap;
    use im_rc::vector;
    use serde::de::DeserializeSeed;
    use serde::Serialize;
    use serde_json::de::StrRead;

    use crate::de::Seed;
    use crate::Record;
    use crate::Value;

    #[test]
    fn serde_i32() {
        let v0 = Value::from(1);
        let s = serde_json::to_string(&v0).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t = hir::TNominal("i32".to_string(), vector![]).into();
        let v1 = Seed(t).deserialize(&mut de).unwrap();
        assert_eq!(v0, v1);
        assert_eq!(s, "1");
    }

    #[test]
    fn serde_vec() {
        let v0 = Value::from(1);
        let v1 = Value::from(2);
        let v2 = Value::from(3);
        let v3 = Value::from(builtins::vec::Vec::from(vec![v0, v1, v2]));
        let s = serde_json::to_string(&v3).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Vec".to_string(), vector![t0]).into();
        let v4 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v3, v4);
        assert_eq!(s, "[1,2,3]");
    }

    #[test]
    fn serde_tuple() {
        let v0 = Value::from(1);
        let v1 = Value::from(2);
        let v2 = Value::from(builtins::string::String::from("Hello"));
        let v3 = Value::from(crate::dynamic::Tuple(vector![v0, v1, v2]));
        let s = serde_json::to_string(&v3).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("i32".to_string(), vector![]).into();
        let t2 = hir::TNominal("String".to_string(), vector![]).into();
        let t3 = hir::TTuple(vector![t0, t1, t2], true).into();
        let v4 = Seed(t3).deserialize(&mut de).unwrap();
        assert_eq!(v3, v4);
        assert_eq!(s, r#"[1,2,"Hello"]"#);
    }

    #[test]
    fn serde_record() {
        let v0 = Value::from(1);
        let v1 = Value::from(builtins::string::String::from("Hello"));
        let v2 = Value::from(Record(hashmap! {
            "a".to_string() => v0,
            "b".to_string() => v1,
        }));
        let s = serde_json::to_string(&v2).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("String".to_string(), vector![]).into();
        let t2 = hir::TRecord(hir::fields_to_row(vector![("a".to_string(), t0), ("b".to_string(), t1),])).into();
        let v3 = Seed(t2).deserialize(&mut de).unwrap();
        assert_eq!(v2, v3);
        assert!((s == r#"{"a":1,"b":"Hello"}"#) || (s == r#"{"b":"Hello","a":1}"#));
    }

    #[test]
    fn serde_dict() {
        let k0 = Value::from(builtins::string::String::from("a"));
        let k1 = Value::from(builtins::string::String::from("b"));
        let v0 = Value::from(1);
        let v1 = Value::from(2);
        let v2 = Value::from(builtins::dict::Dict::from(vec![(k0, v0), (k1, v1)].into_iter().collect::<std::collections::HashMap<_, _>>()));
        let s = serde_json::to_string(&v2).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("String".to_string(), vector![]).into();
        let t1 = hir::TNominal("i32".to_string(), vector![]).into();
        let t2 = hir::TNominal("Dict".to_string(), vector![t0, t1]).into();
        let v3 = Seed(t2).deserialize(&mut de).unwrap();
        assert_eq!(v2, v3);
        assert!((s == r#"{"a":1,"b":2}"#) || (s == r#"{"b":2,"a":1}"#));
    }

    #[test]
    fn serde_array() {
        let v0 = Value::from(1);
        let v1 = Value::from(2);
        let v2 = Value::from(3);
        let v3 = Value::from(crate::dynamic::Array(vector![v0, v1, v2]));
        let s = serde_json::to_string(&v3).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TArray(t0, Some(3)).into();
        let v4 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v3, v4);
        assert_eq!(s, "[1,2,3]");
    }

    #[test]
    fn serde_set() {
        let v0 = Value::from(1);
        let v1 = Value::from(2);
        let v2 = Value::from(builtins::set::Set::from(vec![v0, v1].into_iter().collect::<std::collections::HashSet<_>>()));
        let s = serde_json::to_string(&v2).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Set".to_string(), vector![t0]).into();
        let v3 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v2, v3);
        assert!((s == r#"[1,2]"#) || (s == r#"[2,1]"#));
    }

    #[test]
    fn serde_option_some() {
        let v0 = Value::from(1);
        let v1 = Value::from(builtins::option::Option::some(v0));
        let s = serde_json::to_string(&v1).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Option".to_string(), vector![t0]).into();
        let v2 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v1, v2);
        assert_eq!(s, "1");
    }

    #[test]
    fn serde_option_none() {
        let v0 = Value::from(builtins::option::Option::none());
        let s = serde_json::to_string(&v0).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Option".to_string(), vector![t0]).into();
        let v2 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v0, v2);
        assert_eq!(s, "null");
    }

    #[test]
    fn serde_result_ok() {
        let v0 = Value::from(1);
        let v1 = Value::from(builtins::result::Result::ok(v0));
        let s = serde_json::to_string(&v1).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Result".to_string(), vector![t0]).into();
        let v2 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v1, v2);
        assert_eq!(s, r#"{"Ok":1}"#);
    }

    #[test]
    fn serde_result_err() {
        let v0 = builtins::string::String::from("Hello");
        let v1 = Value::from(builtins::result::Result::error(v0));
        let s = serde_json::to_string(&v1).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Result".to_string(), vector![t0]).into();
        let v2 = Seed::deserialize(Seed(t1), &mut de).unwrap();
        assert_eq!(v1, v2);
        assert_eq!(s, r#"{"Err":"Hello"}"#);
    }

    #[test]
    fn serde_matrix() {
        let v9 = Value::from(crate::dynamic::Matrix::I32(builtins::matrix::Matrix::zeros(vec![2, 2].into())));
        let s = serde_json::to_string(&v9).unwrap();
        let mut de = serde_json::Deserializer::from_str(&s);
        let t0 = hir::TNominal("i32".to_string(), vector![]).into();
        let t1 = hir::TNominal("Matrix".to_string(), vector![t0]).into();
        let v10 = Seed(t1).deserialize(&mut de).unwrap();
        assert_eq!(v9, v10);
        assert_eq!(s, r#"{"v":1,"dim":[2,2],"data":[0,0,0,0]}"#);
    }

    #[test]
    fn serde_type_variable() {
        let mut de = serde_json::Deserializer::from_str("1");
        let t = hir::TVar("a".to_string()).into();
        assert!(Seed(t).deserialize(&mut de).is_err());
    }

    #[test]
    fn serde_type_error() {
        let mut de = serde_json::Deserializer::from_str("1.0");
        let t = hir::TNominal("i32".to_string(), vector![]).into();
        assert!(Seed(t).deserialize(&mut de).is_err());
    }
}

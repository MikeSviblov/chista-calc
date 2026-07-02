use super::arity;
use crate::registry::Registry;
use crate::value::Value;

pub fn register(r: &mut Registry) {
    r.register("Sin", |a, pos| {
        arity(a, 1, "Sin", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.sin()))
    });
    r.register("Cos", |a, pos| {
        arity(a, 1, "Cos", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.cos()))
    });
    r.register("Tan", |a, pos| {
        arity(a, 1, "Tan", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.tan()))
    });
    r.register("Cotan", |a, pos| {
        arity(a, 1, "Cotan", pos)?;
        Ok(Value::Float(1.0 / a[0].as_float(pos)?.tan()))
    });
    r.register("ArcSin", |a, pos| {
        arity(a, 1, "ArcSin", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.asin()))
    });
    r.register("ArcCos", |a, pos| {
        arity(a, 1, "ArcCos", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.acos()))
    });
    r.register("ArcTan", |a, pos| {
        arity(a, 1, "ArcTan", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.atan()))
    });
    r.register("SinH", |a, pos| {
        arity(a, 1, "SinH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.sinh()))
    });
    r.register("CosH", |a, pos| {
        arity(a, 1, "CosH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.cosh()))
    });
    r.register("TanH", |a, pos| {
        arity(a, 1, "TanH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.tanh()))
    });
    r.register("ArcSinH", |a, pos| {
        arity(a, 1, "ArcSinH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.asinh()))
    });
    r.register("ArcCosH", |a, pos| {
        arity(a, 1, "ArcCosH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.acosh()))
    });
    r.register("ArcTanH", |a, pos| {
        arity(a, 1, "ArcTanH", pos)?;
        Ok(Value::Float(a[0].as_float(pos)?.atanh()))
    });
    r.register("DegToRad", |a, pos| {
        arity(a, 1, "DegToRad", pos)?;
        Ok(Value::Float(a[0].as_float(pos)? * std::f64::consts::PI / 180.0))
    });
    r.register("RadToDeg", |a, pos| {
        arity(a, 1, "RadToDeg", pos)?;
        Ok(Value::Float(a[0].as_float(pos)? * 180.0 / std::f64::consts::PI))
    });
}

#[cfg(test)]
mod tests {
    use crate::registry::Registry;
    use crate::value::Value;
    fn callf(name: &str, x: f64) -> f64 {
        match Registry::with_builtins().get(name).unwrap()(&[Value::Float(x)], 0).unwrap() { Value::Float(v) => v, _ => panic!() }
    }
    #[test]
    fn trig_values() {
        assert!(callf("Sin", 0.0).abs() < 1e-12);
        assert!((callf("Cos", 0.0) - 1.0).abs() < 1e-12);
        assert!((callf("DegToRad", 180.0) - std::f64::consts::PI).abs() < 1e-12);
        assert!((callf("RadToDeg", std::f64::consts::PI) - 180.0).abs() < 1e-9);
    }
}

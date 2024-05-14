struct ConnCurve {
    curves: Vec<QuadraticBezierCurve>,
    // the prefix sum of the length of each curve
    lensSum: Vec<f64>,
}

impl ConnCurve {
    
}
/// What should the type of _function be?
pub fn map<IN, OUT>(input: Vec<IN>, mut fun: impl FnMut (IN) -> OUT) -> Vec<OUT> {
    let mut result = vec![];
    for item in input {
        result.push( fun(item));
    }
    result
}

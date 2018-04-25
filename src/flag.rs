pub enum Flag {
    Yield, // remove and yield
    Return, // remove, yield and stop iteration
    Continue,    // keep
    Break,   // keep and stop iteration
}

pub enum MoveFlag<T> {
    Yield(T), // remove and yield
    Return(T), // remove, yield and stop iteration
    Continue,    // keep
    Break,   // keep and stop iteration
}

use std::thread;

#[derive(Debug)]
pub enum Error {
    NoResult,
}

pub trait State<T: State<T> + Clone> {
    fn merge(&self, b: &T) -> T;
}
pub trait Task<T: State<T> + Clone> {
    fn name(&self) -> &str;
    fn execute(&self, state: T) -> Result<T, Error>;
}

pub trait Condition<T: State<T> + Clone> {
    fn name(&self) -> &str;
    fn execute(&self, state: T) -> Result<bool, Error>;
}

pub trait WithName {
    fn with_name(&mut self, n: String) -> &Self;
}

pub struct SequenceTask<'a, T: State<T> + Clone> {
    n: String,
    tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T> + Clone> Task<T> for SequenceTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        let mut state = state;
        for task in &self.tasks {
            state = task.execute(state)?;
        }
        Ok(state)
    }
}

impl<'a, T: State<T> + Clone> WithName for SequenceTask<'a, T> {
    fn with_name(&mut self, n: String) -> &Self {
        self.n = n;
        self
    }
}

pub struct OrTask<'a, T: State<T> + Clone> {
    n: String,
    tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T> + Clone> Task<T> for OrTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        for task in &self.tasks {
            let res = task.execute(state.clone());
            if res.is_ok() {
                return res;
            }
        }
        Err(Error::NoResult)
    }
}

impl<'a, T: State<T> + Clone> WithName for OrTask<'a, T> {
    fn with_name(&mut self, n: String) -> &Self {
        self.n = n;
        self
    }
}

pub struct ConcurrentTask<'a, T: State<T> + Clone> {
    n: String,
    tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T> + Clone> Task<T> for ConcurrentTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        for task in &self.tasks {}
        Err(Error::NoResult)
    }
}

impl<'a, T: State<T> + Clone> WithName for ConcurrentTask<'a, T> {
    fn with_name(&mut self, n: String) -> &Self {
        self.n = n;
        self
    }
}

pub struct IfTask<'a, T: State<T> + Clone> {
    n: String,
    condition: &'a dyn Condition<T>,
    then_do: &'a dyn Task<T>,
    default_do: &'a dyn Task<T>,
}

impl<'a, T: State<T> + Clone> Task<T> for IfTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        let condition = self.condition.execute(state.clone())?;
        if condition {
            self.then_do.execute(state)
        } else {
            self.default_do.execute(state)
        }
    }
}

impl<'a, T: State<T> + Clone> WithName for IfTask<'a, T> {
    fn with_name(&mut self, n: String) -> &Self {
        self.n = n;
        self
    }
}

impl<'a, T: State<T> + Clone> IfTask<'a, T> {
    pub fn with_default(&mut self, task: &'a dyn Task<T>) -> &Self {
        self.default_do = task;
        self
    }
}

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
    fn with_name(&mut self, n: &str) -> Self;
}

#[derive(Clone)]
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
    fn with_name(&mut self, n: &str) -> Self {
        self.n = n.to_string();
        self.clone()
    }
}

#[derive(Clone)]
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
    fn with_name(&mut self, n: &str) -> Self {
        self.n = n.to_string();
        self.clone()
    }
}

#[derive(Clone)]
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
    fn with_name(&mut self, n: &str) -> Self {
        self.n = n.to_string();
        self.clone()
    }
}

#[derive(Clone)]
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
    fn with_name(&mut self, n: &str) -> Self {
        self.n = n.to_string();
        self.clone()
    }
}

impl<'a, T: State<T> + Clone> IfTask<'a, T> {
    pub fn with_default(&mut self, task: &'a dyn Task<T>) -> &Self {
        self.default_do = task;
        self
    }
}

pub struct TaskImpl<'a, T: State<T> + Clone> {
    n: &'a str,
    method: &'a dyn Fn(T) -> Result<T, Error>,
}

impl<'a, T: State<T> + Clone> Task<T> for TaskImpl<'a, T> {
    fn name(&self) -> &str {
        self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        (self.method)(state)
    }
}

pub fn new_task<'a, T: State<T> + Clone>(
    n: &'a str,
    method: &'a dyn Fn(T) -> Result<T, Error>,
) -> TaskImpl<'a, T> {
    TaskImpl {
        n: n,
        method: method,
    }
}

pub fn sequence<'a, T: State<T> + Clone>(tasks: Vec<&'a dyn Task<T>>) -> SequenceTask<'a, T> {
    SequenceTask {
        n: "".to_string(),
        tasks: tasks,
    }
}

pub fn or<'a, T: State<T> + Clone>(tasks: Vec<&'a dyn Task<T>>) -> OrTask<'a, T> {
    OrTask {
        n: "".to_string(),
        tasks: tasks,
    }
}

mod test {
    use super::*;
    #[derive(Debug)]
    struct TestState {
        num: i32,
    }
    impl State<TestState> for TestState {
        fn merge(&self, b: &TestState) -> TestState {
            TestState { num: self.num }
        }
    }
    impl Clone for TestState {
        fn clone(&self) -> Self {
            TestState { num: self.num }
        }
    }

    #[test]
    fn test_sequence() {
        let task1 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 1 })
        });
        let task2 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 2 })
        });
        let task3 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 3 })
        });
        let seq_task = sequence(vec![&task1, &task2, &task3]).with_name("seq_task");
        let res = seq_task.execute(TestState { num: 100 });
        print!("{:?}", res);
    }

    #[test]
    fn test_or() {
        let task1 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 1 })
        });
        let task2 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 2 })
        });
        let task3 = new_task("task1", &|a: TestState| -> Result<TestState, Error> {
            Ok(TestState { num: a.num + 3 })
        });
        let or_task = or(vec![&task1, &task2, &task3]).with_name("or_task");
        let res = or_task.execute(TestState { num: 100 });
        print!("{:?}", res);
    }
}

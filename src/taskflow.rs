use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    NoResult,
}

pub trait State<T: State<T>>: Clone {
    fn merge(&self, b: &T) -> T;
}
pub trait Task<T: State<T>> {
    fn name(&self) -> &str;
    fn execute(&self, state: T) -> Result<T, Error>;
}

pub trait Condition<T: State<T>> {
    fn name(&self) -> &str;
    fn execute(&self, state: T) -> Result<bool, Error>;
}

pub trait WithName {
    fn with_name(self, n: &str) -> Self;
}

pub struct SequenceTask<'a, T: State<T>> {
    pub n: String,
    pub tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T>> Task<T> for SequenceTask<'a, T> {
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

impl<'a, T: State<T>> WithName for SequenceTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

pub struct OrTask<'a, T: State<T>> {
    n: String,
    tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T>> Task<T> for OrTask<'a, T> {
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

impl<'a, T: State<T>> WithName for OrTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

pub struct ConcurrentTask<'a, T: State<T>> {
    n: String,
    tasks: Vec<&'a dyn Task<T>>,
}

impl<'a, T: State<T>> Task<T> for ConcurrentTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        for task in &self.tasks {}
        Err(Error::NoResult)
    }
}

impl<'a, T: State<T>> WithName for ConcurrentTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

pub struct IfTask<'a, T: State<T>> {
    n: String,
    condition: &'a dyn Condition<T>,
    then_do: &'a dyn Task<T>,
    default_do: Option<&'a dyn Task<T>>,
}

impl<'a, T: State<T>> Task<T> for IfTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        let condition = self.condition.execute(state.clone())?;
        if condition {
            self.then_do.execute(state)
        } else if self.default_do.is_some() {
            self.default_do.unwrap().execute(state)
        } else {
            Err(Error::NoResult)
        }
    }
}

impl<'a, T: State<T>> WithName for IfTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

impl<'a, T: State<T>> IfTask<'a, T> {
    pub fn with_default(&mut self, task: &'a dyn Task<T>) -> &Self {
        self.default_do = Some(task);
        self
    }
}

pub struct LoopTask<'a, T: State<T>> {
    n: String,
    condition: &'a dyn Condition<T>,
    task: &'a dyn Task<T>,
}

impl<'a, T: State<T>> Task<T> for LoopTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        let mut state = state;
        while self.condition.execute(state.clone())? {
            state = self.task.execute(state)?;
        }
        Ok(state)
    }
}

impl<'a, T: State<T>> WithName for LoopTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

pub struct PromiseTask<'a, T: State<T>> {
    n: String,
    init_task: &'a dyn Task<T>,
    other_tasks: Vec<Option<&'a dyn Task<T>>>,
}

impl<'a, T: State<T>> Task<T> for PromiseTask<'a, T> {
    fn name(&self) -> &str {
        &self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        let mut state = state;
        for task in &self.other_tasks {
            if task.is_some() {
                state = task.unwrap().execute(state)?;
            }
        }
        Ok(state)
    }
}

impl<'a, T: State<T>> WithName for PromiseTask<'a, T> {
    fn with_name(mut self, n: &str) -> Self {
        self.n = n.to_string();
        self
    }
}

pub struct TaskImpl<'a, T: State<T>> {
    n: &'a str,
    method: &'a dyn Fn(T) -> Result<T, Error>,
}

impl<'a, T: State<T>> Task<T> for TaskImpl<'a, T> {
    fn name(&self) -> &str {
        self.n
    }
    fn execute(&self, state: T) -> Result<T, Error> {
        (self.method)(state)
    }
}

pub fn new_task<'a, T: State<T>>(
    n: &'a str,
    method: &'a dyn Fn(T) -> Result<T, Error>,
) -> TaskImpl<'a, T> {
    TaskImpl {
        n: n,
        method: method,
    }
}

pub fn sequence_task<'a, T: State<T>>(tasks: Vec<&'a dyn Task<T>>) -> SequenceTask<'a, T> {
    SequenceTask {
        n: "".to_string(),
        tasks: tasks,
    }
}

pub fn or_task<'a, T: State<T>>(tasks: Vec<&'a dyn Task<T>>) -> OrTask<'a, T> {
    OrTask {
        n: "".to_string(),
        tasks: tasks,
    }
}

pub fn if_task<'a, T: State<T>>(
    condition: &'a dyn Condition<T>,
    then_do: &'a dyn Task<T>,
) -> IfTask<'a, T> {
    IfTask {
        n: "".to_string(),
        condition: condition,
        then_do: then_do,
        default_do: None,
    }
}

pub fn loop_task<'a, T: State<T>>(
    condition: &'a dyn Condition<T>,
    task: &'a dyn Task<T>,
) -> LoopTask<'a, T> {
    LoopTask {
        n: "".to_string(),
        condition: condition,
        task: task,
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
        let seq_task = sequence_task(vec![&task1, &task2, &task3]).with_name("seq_task");
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
        let or_task = or_task(vec![&task1, &task2, &task3]).with_name("or_task");
        let res = or_task.execute(TestState { num: 100 });
        print!("{:?}", res);
    }
}

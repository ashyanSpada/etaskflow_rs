use crate::taskflow::*;
use dot_rs::*;

fn transform_task<'a, T: State<T>>(
    task: &'a dyn Task<T>,
) -> (&'a dyn Node, &'a dyn Node, &'a dyn Stmt) {
    match task {
        SequenceTask => transform_sequence_task(task),
    }
}

fn transform_sequence_task<'a, T: State<T>>(
    task: &'a SequenceTask<'a, T>,
) -> (&'a dyn Node, &'a dyn Node, &'a dyn Stmt) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut stmts = Vec::new();
    for t in &task.tasks {
        let (n1, n2, stmt) = transform_task(t);
        nodes.push(n1);
        nodes.push(n2);
        edges.push(
            new_edge(n1)
                .with_attribute("label", "next")
                .with_attribute("style", "dashed"),
        );
        edges.push(
            new_edge(n2)
                .with_attribute("label", "next")
                .with_attribute("style", "dashed"),
        );
        edges.push(
            new_edge(n1)
                .with_attribute("label", "next")
                .with_attribute("style", "dashed"),
        );
        stmts.push(stmt);
    }
    (nodes[0], nodes[1], stmts[0])
}

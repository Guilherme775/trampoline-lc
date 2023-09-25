use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum Expr {
    Var(String),
    Abs(String, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    VClosure(Context, String, Box<Expr>),
}

type Context = HashMap<String, Value>;

enum Trampoline {
    Continue(Box<dyn FnOnce() -> Trampoline>),
    Complete(Value),
}

impl Trampoline {
    fn run(self) -> Value {
        let mut current_trampoline = self;
        loop {
            match current_trampoline {
                Trampoline::Complete(value) => return value,
                Trampoline::Continue(func) => current_trampoline = func(),
            }
        }
    }
}

fn eval_with_trampoline(expr: Expr, context: Context) -> Trampoline {
    match expr {
        Expr::Var(name) => Trampoline::Complete(
            context
                .get(&name)
                .cloned()
                .unwrap_or_else(|| panic!("Variable {} not found", name)),
        ),
        Expr::Abs(param, body) => Trampoline::Complete(Value::VClosure(context, param, body)),
        Expr::App(f, arg) => Trampoline::Continue(Box::new(move || {
            let func_value_tramp = eval_with_trampoline(*f.clone(), context.clone());
            let arg_value_tramp = eval_with_trampoline(*arg.clone(), context.clone());
            match func_value_tramp.run() {
                Value::VClosure(ctx, param, body) => {
                    let arg_value = arg_value_tramp.run();
                    let mut new_ctx = ctx;
                    new_ctx.insert(param, arg_value);
                    eval_with_trampoline(*body, new_ctx)
                }
            }
        })),
    }
}

fn eval_without_trampoline(expr: Expr, context: HashMap<String, Value>) -> Value {
    match expr {
        Expr::Var(name) => match context.get(&name) {
            Some(value) => value.clone(),
            None => panic!("Variable {} not found", name),
        },
        Expr::Abs(param, body) => Value::VClosure(context, param, body),
        Expr::App(f, arg) => {
            let Value::VClosure(ctx, param, body) = eval_without_trampoline(*f, context.clone());
            let arg_value = eval_without_trampoline(*arg, context.clone());

            let mut new_ctx = ctx;

            new_ctx.insert(param, arg_value);

            eval_without_trampoline(*body, new_ctx)
        }
    }
}

fn main() {
    println!("Trampoline");
}

#[test]
pub fn check_results() {
    let two = Expr::Abs(
        "f".to_string(),
        Box::new(Expr::Abs(
            "x".to_string(),
            Box::new(Expr::App(
                Box::new(Expr::Var("f".to_string())),
                Box::new(Expr::App(
                    Box::new(Expr::Var("f".to_string())),
                    Box::new(Expr::Var("x".to_string())),
                )),
            )),
        )),
    );

    let pred = Expr::App(
        Box::new(Expr::Abs(
            "n".to_string(),
            Box::new(Expr::App(
                Box::new(Expr::App(
                    Box::new(Expr::Var("n".to_string())),
                    Box::new(Expr::Abs(
                        "g".to_string(),
                        Box::new(Expr::Abs(
                            "h".to_string(),
                            Box::new(Expr::App(
                                Box::new(Expr::Var("h".to_string())),
                                Box::new(Expr::App(
                                    Box::new(Expr::Var("g".to_string())),
                                    Box::new(Expr::Var("f".to_string())),
                                )),
                            )),
                        )),
                    )),
                )),
                Box::new(Expr::Abs(
                    "u".to_string(),
                    Box::new(Expr::Var("x".to_string())),
                )),
            )),
        )),
        Box::new(two.clone()),
    );

    assert_eq!(
        eval_with_trampoline(pred.clone(), HashMap::new()).run(),
        eval_without_trampoline(pred, HashMap::new())
    );
}

// Uncomment this if you want to see that without trampoline stack overflow will happen
// #[test]
// fn stack_overflow() {
//     let looping_expr = Expr::App(
//         Box::new(Expr::Abs(
//             "x".to_string(),
//             Box::new(Expr::App(
//                 Box::new(Expr::Var("x".to_string())),
//                 Box::new(Expr::Var("x".to_string())),
//             )),
//         )),
//         Box::new(Expr::Abs(
//             "x".to_string(),
//             Box::new(Expr::App(
//                 Box::new(Expr::Var("x".to_string())),
//                 Box::new(Expr::Var("x".to_string())),
//             )),
//         )),
//     );

//     eval_without_trampoline(looping_expr, HashMap::new());
// }

// This will never stop btw
#[test]
fn not_stack_overflow() {
    let looping_expr = Expr::App(
        Box::new(Expr::Abs(
            "x".to_string(),
            Box::new(Expr::App(
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("x".to_string())),
            )),
        )),
        Box::new(Expr::Abs(
            "x".to_string(),
            Box::new(Expr::App(
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("x".to_string())),
            )),
        )),
    );

    eval_with_trampoline(looping_expr, HashMap::new()).run();
}

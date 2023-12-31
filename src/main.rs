// TODO: Get better Backspace key icon
use yew::{
    html,
    classes,
    BaseComponent,
    Component,
    Context,
    Html,
    Classes,
    KeyboardEvent,
    function_component,
    Callback,
    Properties,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use gloo_events::EventListener;
use gloo_utils::window;
use yew_hooks::use_clipboard;

#[derive(Clone, Copy)]
pub enum Msg {
    Calculate,
    ClickNumber(f64),
    ClickOperator(Operator),
    ClickDot,
    Backspace,
    Clear,
    LoadFromHistory(usize),
}
#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    Mul,
    Div,
    Add,
    Sub,
}
impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mul => write!(f, " * "),
            Self::Div => write!(f, " / "),
            Self::Add => write!(f, " + "),
            Self::Sub => write!(f, " - "),
        }
    }
}

fn number_button<COMP>(num: f64, styles: Classes, ctx: &Context<COMP>) -> Html 
where
    COMP: BaseComponent,
    <COMP as yew::BaseComponent>::Message: std::convert::From<Msg>,
{
    html! {
        <button
            class={classes!("bg-slate-400", "text-black", "font-bold", "py-2", "px-4", "h-20", "rounded", styles)}
            onclick={ctx.link().callback(move |_| Msg::ClickNumber(num))}>
            { format!("{num}") }
        </button>
    }
}

fn button<COMP>(msg: Msg, styles: Classes, ctx: &Context<COMP>) -> Html
where
    COMP: BaseComponent,
    <COMP as yew::BaseComponent>::Message: std::convert::From<Msg>,
{
    html! {
        <button class={classes!("bg-slate-400", "text-black", "font-bold", "py-2", "px-4", "h-20", "rounded", styles)}
            onclick={ctx.link().callback(move |_| msg)}>
            {
                match msg {
                    Msg::Calculate => "=".to_string(),
                    Msg::ClickOperator(o) => {
                        o.to_string().trim().to_string()
                    },
                    Msg::ClickDot => ".".to_string(),
                    Msg::Backspace => "B".to_string(),
                    Msg::Clear => "AC".to_string(),
                    _ => String::new(),
                }
            }
        </button>
    }
}

#[derive(Properties, PartialEq)]
struct DisplayProps {
    calculation: (String, Operator, String, String),
    set_number_1: bool,
}

#[function_component]
fn CalculatorDisplay(props: &DisplayProps) -> Html {
    let clipboard = use_clipboard();
    let calculation = &props.calculation;
    let onclick = {
        let clipboard = clipboard.clone();
        let res = calculation.3.clone();
        Callback::from(move |_| {
            clipboard.write_text(res.to_string());
        })
    };

    html! {
        <>
            <p class={classes!("text-xl", "bg-slate-200", "rounded-t-lg", "px-2", "py-1")}>
                { &calculation.0 }
                {
                    if !props.set_number_1 {
                        calculation.1.to_string() + &calculation.2
                    } else {
                        "".to_string()
                    }
                }
            </p>
            <p class={classes!("text-right", "bg-slate-200", "rounded-b-lg", "mb-3", "px-2", "py-1", "hover:cursor-pointer", "hover:text-green-800")} // Replace with function component
                {onclick}>
                { "= " }{ &calculation.3 }
            </p>
        </>
    }
}

pub struct Calculator {
    result: String,
    number_1: String,
    number_2: String,
    operator: Operator,
    set_number_1: bool,
    fragile_input: bool,
    history: Vec<(f64, Operator, f64, f64)>,
    _keydown_listener: EventListener,
}
impl Calculator {
    pub fn calculate(&mut self) {
        let num_1: f64 = self.number_1.parse().unwrap();
        let num_2: f64 = self.number_2.parse().unwrap();
        let res = match self.operator {
            Operator::Mul => num_1 * num_2,
            Operator::Div => num_1 / num_2,
            Operator::Add => num_1 + num_2,
            Operator::Sub => num_1 - num_2,
        };

        self.history.push((num_1, self.operator, num_2, res));

        if res.is_nan() || res.is_infinite() {
            self.result = "Error".to_string();
            self.number_1 = "0".to_string();
        } else {
            self.result = res.to_string();
            self.number_1 = res.to_string();
        }
        self.set_number_1 = true;
        self.fragile_input = true;
    }
}

impl Component for Calculator {
    type Message = Msg;
    type Properties = ();
    
    fn create(ctx: &Context<Self>) -> Self {
        let listener = EventListener::new(&window(), "keydown", {
            let link = ctx.link().clone();
            move |event| {
                let event: &KeyboardEvent = event.dyn_ref::<KeyboardEvent>().unwrap_throw();
            
                let msg = match event {
                    k if (48..=57).contains(&k.key_code()) && !k.shift_key() => { // Check if numeric (top row) and no shift key
                        let n = (k.key_code() - 48) as f64; // Convert code to numeric value
                        Some(Msg::ClickNumber(n))
                    },
                    k if (96..=105).contains(&k.key_code()) => { // Check if numeric (num pad)
                        let n = (k.key_code() - 96) as f64; // Convert code to numeric value
                        Some(Msg::ClickNumber(n))
                    },
                    // Multiply key
                    o if o.key_code() == 106 // num pad
                        || o.key_code() == 56 && o.shift_key() => { // top row (shift + 8)
                        Some(Msg::ClickOperator(Operator::Mul))
                    },
                    // Divide key
                    o if o.key_code() == 111 // num pad
                        || o.key_code() == 191 && !o.shift_key() => { // slash (no shift)
                        Some(Msg::ClickOperator(Operator::Div))
                    },
                    // Add key
                    o if o.key_code() == 107 // num pad
                        || o.key_code() == 187 && o.shift_key() => { // top row (shift + =)
                        Some(Msg::ClickOperator(Operator::Add))
                    },
                    // Subtract key
                    o if o.key_code() == 109 // num pad
                        || o.key_code() == 189 && !o.shift_key() => { // minus (no shift)
                        Some(Msg::ClickOperator(Operator::Sub))
                    },
                    // Dot key
                    o if o.key_code() == 110 // num pad
                        || o.key_code() == 190 && !o.shift_key() => { // period (no shift)
                        Some(Msg::ClickDot)
                    },
                    // Enter / Equal key
                    o if o.key_code() == 13 // num pad (and regular keyboard) enter
                        || o.key_code() == 187 && !o.shift_key() => { // equal (no shift)
                        Some(Msg::Calculate)
                    },
                    // Backspace key
                    o if o.key_code() == 8 => {
                        Some(Msg::Backspace)
                    },
                    // Clear (delete key)
                    o if o.key_code() == 46 => {
                        Some(Msg::Clear)
                    },
                    // TODO: Detect all non-num-lock versions of numbers
                    _ => None,
                };

                if let Some(m) = msg {
                    link.send_message(m);
                }
            }
        });
        Self {
            result: "0".to_string(),
            number_1: 0.0.to_string(),
            number_2: 0.0.to_string(),
            operator: Operator::Add,
            set_number_1: true,
            fragile_input: false,
            history: vec![],
            _keydown_listener: listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Calculate => self.calculate(),
            Msg::ClickNumber(n) => {
                if self.set_number_1 {
                    if (self.number_1.parse::<f64>().unwrap() == 0. && !self.number_1.contains('.')) || self.fragile_input {
                        self.number_1 = n.to_string();
                    } else {
                        self.number_1.push_str(&n.to_string());
                    }
                    self.number_2 = "0".to_string();
                    self.operator = Operator::Add;
                } else if self.number_2.parse::<f64>().unwrap() == 0. && !self.number_2.contains('.') {
                    self.number_2 = n.to_string();
                } else {
                    self.number_2.push_str(&n.to_string());
                }
                self.fragile_input = false;
            },
            Msg::ClickOperator(o) => {
                if !self.fragile_input && !self.set_number_1 {
                    self.calculate()
                }
                self.number_2 = "0".to_string();
                self.operator = o;
                self.set_number_1 = false;
                self.fragile_input = true;
            },
            Msg::ClickDot => {
                if self.set_number_1 {
                    if !self.number_1.contains('.') {
                        self.number_1.push('.');
                    }
                } else if !self.number_2.contains('.') {
                    self.number_2.push('.');
                }
                self.fragile_input = false;
            },
            Msg::Backspace => {
                if self.set_number_1 {
                    if self.number_1.len() <= 1 || self.fragile_input {
                        self.number_1 = "0".to_string();
                    } else {
                        self.number_1.pop();
                    }
                } else if self.number_2.parse::<f64>().unwrap() == 0. {
                    self.set_number_1 = true;
                } else if self.number_2.len() <= 1 {
                    self.number_2 = "0".to_string();
                } else {
                    self.number_2.pop();
                }
                self.fragile_input = false;
            },
            Msg::Clear => { // Reset
                self.number_1 = "0".to_string();
                self.number_2 = "0".to_string();
                self.set_number_1 = true;
                self.fragile_input = false;
            },
            Msg::LoadFromHistory(i) => {
                match self.history.get(i) {
                    None => (),
                    Some(calc) => {
                        self.number_1 = calc.0.to_string();
                        self.operator = calc.1;
                        self.number_2 = calc.2.to_string();
                        self.result = calc.3.to_string();

                        self.set_number_1 = false;
                        self.fragile_input = false;
                    }
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let no_styles = Classes::new();
        let calculation = (
            self.number_1.clone(),
            self.operator,
            self.number_2.clone(),
            self.result.clone(),
        );
        let set_number_1 = self.set_number_1;
        html! {
            <div class={classes!("font-mono", "mx-auto", "sm:mt-4", "sm:container", "sm:w-full", "md:w-1/2", "lg:w-1/3", "xl:w-1/4")}>
                <div class={classes!("bg-slate-500", "p-5", "sm:rounded-t-xl")}>
                    <div>
                        <CalculatorDisplay {calculation} {set_number_1} />
                    </div>
                    <div class="grid grid-cols-5 gap-3"> // Button panel
                        { number_button(1., no_styles.clone(), ctx) }
                        { number_button(2., no_styles.clone(), ctx) }
                        { number_button(3., no_styles.clone(), ctx) }
                        { button(Msg::Backspace, Classes::from("bg-slate-600"), ctx) }
                        { button(Msg::Clear, Classes::from("bg-slate-600"), ctx) }

                        { number_button(4., no_styles.clone(), ctx) }
                        { number_button(5., no_styles.clone(), ctx) }
                        { number_button(6., no_styles.clone(), ctx) }
                        { button(Msg::ClickOperator(Operator::Mul), Classes::from("bg-slate-600"), ctx) }
                        { button(Msg::ClickOperator(Operator::Div), Classes::from("bg-slate-600"), ctx) }

                        { number_button(7., no_styles.clone(), ctx) }
                        { number_button(8., no_styles.clone(), ctx) }
                        { number_button(9., no_styles.clone(), ctx) }
                        { button(Msg::ClickOperator(Operator::Add), Classes::from("bg-slate-600"), ctx) }
                        { button(Msg::ClickOperator(Operator::Sub), Classes::from("bg-slate-600"), ctx) }

                        { number_button(0., Classes::from("col-span-2"), ctx) }
                        { button(Msg::ClickDot, no_styles.clone(), ctx) }
                        { button(Msg::Calculate, Classes::from("col-span-2 bg-slate-600"), ctx) }
                    </div>
                </div>
                <div class={classes!("bg-slate-200", "p-5", "sm:rounded-b-xl")}>
                    <h1 class={classes!("text-xl")}>{ "History" }</h1>
                    {
                        self.history.iter().enumerate().map(|index_calculation| {
                            let (i, c) = index_calculation;
                            html! {
                                <p class={classes!("cursor-pointer", "w-max", "text-black/60", "hover:text-black")}
                                    onclick={ctx.link().callback(move |_| Msg::LoadFromHistory(i))}>
                                    { c.0 }{ c.1 }{ c.2 }{ " = " }{ c.3 }
                                </p>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<Calculator>::new().render();
}

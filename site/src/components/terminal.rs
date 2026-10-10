//! A terminal that types the whole journey, from `cargo install paquets` to
//! production, when it enters the screen.

use gloo_timers::callback::Interval;
use web_sys::HtmlElement;
use yew::prelude::*;

use super::ui::Icon;
use crate::dom;
use crate::i18n::use_lang;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Typed character by character after the prompt.
    Command,
    /// Compiler output, dimmed.
    Dim,
    /// A result, highlighted.
    Done,
}

struct Line {
    kind: Kind,
    text: String,
    /// The step of the journey it belongs to (`None` before the first one).
    step: Option<usize>,
}

fn script(out: &[&str; 4]) -> Vec<Line> {
    let line = |kind, text: &str, step| Line {
        kind,
        text: text.to_owned(),
        step,
    };
    vec![
        line(Kind::Command, "cargo install paquets", None),
        line(Kind::Dim, "   Compiling paquets v0.1.0", None),
        line(Kind::Done, &format!("✓ {}", out[0]), None),
        line(Kind::Command, "paquets new my-app", Some(0)),
        line(Kind::Done, &format!("📦 {}", out[1]), Some(0)),
        line(Kind::Command, "cd my-app && paquets dev", Some(1)),
        line(Kind::Dim, "   Compiling my-app v0.1.0", Some(1)),
        line(Kind::Done, &format!("➜ {}", out[2]), Some(1)),
        line(Kind::Command, "git push origin my-feature", Some(2)),
        line(Kind::Dim, "   CI: fmt · clippy · test · build", Some(2)),
        line(Kind::Done, &format!("✓ {}", out[3]), Some(3)),
    ]
}

/// Ticks (of 34 ms) spent on a line before the next one starts.
fn duration(line: &Line) -> usize {
    match line.kind {
        Kind::Command => line.text.chars().count() + 14,
        Kind::Dim => 9,
        Kind::Done => 16,
    }
}

/// What is visible at `tick`: the number of complete lines, and the number
/// of characters typed on the line after them.
fn visible(lines: &[Line], tick: usize) -> (usize, usize) {
    let mut left = tick;
    for (i, line) in lines.iter().enumerate() {
        let d = duration(line);
        if left < d {
            let typed = match line.kind {
                Kind::Command => left.min(line.text.chars().count()),
                _ => 0,
            };
            return (i, typed);
        }
        left -= d;
    }
    (lines.len(), 0)
}

#[derive(Properties, PartialEq)]
pub struct TerminalProps {
    /// Told which step of the journey the terminal is at.
    #[prop_or_default]
    pub on_step: Callback<Option<usize>>,
}

#[component]
pub fn Terminal(props: &TerminalProps) -> Html {
    let t = use_lang().lang.texts();
    let lines = script(&t.term_out);
    let total: usize = lines.iter().map(duration).sum();
    let tick = use_state(|| 0usize);
    let started = use_state(|| false);
    let run = use_state(|| 0u32);
    let node = use_node_ref();

    {
        let (started, node) = (started.clone(), node.clone());
        use_effect_with((), move |_| {
            let observer = node
                .cast::<HtmlElement>()
                .map(|el| dom::on_first_visible(&el, 0.3, move || started.set(true)));
            move || {
                if let Some(o) = observer {
                    o.disconnect();
                }
            }
        });
    }
    {
        let tick = tick.clone();
        use_effect_with((*started, *run), move |&(started, _)| {
            let interval = if !started {
                None
            } else if dom::reduced_motion() {
                tick.set(total);
                None
            } else {
                let mut n = 0usize;
                tick.set(0);
                Some(Interval::new(34, move || {
                    if n < total {
                        n += 1;
                        tick.set(n);
                    }
                }))
            };
            move || drop(interval)
        });
    }

    let (done, typed) = visible(&lines, *tick);
    let finished = done >= lines.len();
    let step = lines
        .get(done.min(lines.len() - 1))
        .and_then(|l| l.step)
        .filter(|_| *started);
    {
        let on_step = props.on_step.clone();
        use_effect_with(step, move |step| on_step.emit(*step));
    }
    let replay = {
        let run = run.clone();
        Callback::from(move |_: MouseEvent| run.set(*run + 1))
    };

    let prompt = html! { <span class="term__prompt">{ "➜ " }<span class="term__dir">{ "~" }</span>{ " $ " }</span> };
    html! {
        <div class="window terminal" ref={node}>
            <div class="window__bar">
                <span></span><span></span><span></span>
                <em><Icon name="terminal" />{ "zsh — my-app" }</em>
                <button class={classes!("term__replay", finished.then_some("is-visible"))}
                        onclick={replay} tabindex={if finished { "0" } else { "-1" }}>
                    <Icon name="replay" /><span>{ t.term_replay }</span>
                </button>
            </div>
            <pre class="code term" aria-label={lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n")}>
                { for lines.iter().take(done).map(|line| match line.kind {
                    Kind::Command => html! { <span class="term__line">{ prompt.clone() }<span class="term__cmd">{ line.text.clone() }</span></span> },
                    Kind::Dim => html! { <span class="term__line term__line--dim">{ line.text.clone() }</span> },
                    Kind::Done => html! { <span class="term__line term__line--done">{ line.text.clone() }</span> },
                }) }
                if let Some(line) = lines.get(done) {
                    if line.kind == Kind::Command {
                        <span class="term__line">
                            { prompt.clone() }
                            <span class="term__cmd">{ line.text.chars().take(typed).collect::<String>() }</span>
                            <span class="caret" aria-hidden="true"></span>
                        </span>
                    } else {
                        <span class="term__line term__line--wait"><span class="term__spin" aria-hidden="true"></span></span>
                    }
                } else {
                    <span class="term__line">{ prompt }<span class="caret" aria-hidden="true"></span></span>
                }
            </pre>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_the_script_in_order() {
        let lines = script(&["a", "b", "c", "d"]);
        assert_eq!(visible(&lines, 0), (0, 0));
        assert_eq!(visible(&lines, 5), (0, 5));
        let first = duration(&lines[0]);
        assert_eq!(visible(&lines, first), (1, 0));
        let total: usize = lines.iter().map(duration).sum();
        assert_eq!(visible(&lines, total), (lines.len(), 0));
        // Every step of the journey appears, in order.
        let steps: Vec<usize> = lines.iter().filter_map(|l| l.step).collect();
        assert!(steps.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(steps.last(), Some(&3));
    }
}

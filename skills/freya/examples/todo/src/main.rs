use freya::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app).with_size(560., 620.)))
}

#[derive(Clone, PartialEq)]
struct Task {
    id: usize,
    title: String,
    done: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Active,
    Done,
}

impl Filter {
    fn matches(self, task: &Task) -> bool {
        match self {
            Self::All => true,
            Self::Active => !task.done,
            Self::Done => task.done,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Done => "Done",
        }
    }
}

fn app() -> impl IntoElement {
    use_init_theme(light_theme);

    let draft = use_state(String::new);
    let tasks = use_state(Vec::<Task>::new);
    let next_id = use_state(|| 0usize);
    let mut filter = use_state(|| Filter::All);

    // Deriving in render is cheaper than a memo for values this small.
    let selected = *filter.read();
    let remaining = tasks.read().iter().filter(|t| !t.done).count();
    let can_add = !draft.read().trim().is_empty();

    let add_task = move |_| {
        let title = draft.read().trim().to_owned();
        if title.is_empty() {
            return;
        }
        let id = *next_id.read();
        *next_id.write() += 1;
        tasks.write().push(Task {
            id,
            title,
            done: false,
        });
        draft.write().clear();
    };

    rect()
        .expanded()
        // Required so `Size::flex` children share space.
        .content(Content::Flex)
        .theme_background()
        .theme_color()
        .padding(20.)
        .spacing(12.)
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(label().text("My tasks").font_size(26.))
                .child(label().text(format!("{remaining} left")).font_size(14.)),
        )
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(Input::new(draft).placeholder("What needs doing?"))
                .child(
                    Button::new()
                        .enabled(can_add)
                        .on_press(add_task)
                        .child("Add"),
                ),
        )
        .child(
            rect()
                .horizontal()
                .spacing(6.)
                .children([Filter::All, Filter::Active, Filter::Done].into_iter().map(|f| {
                    ButtonSegment::new()
                        .selected(selected == f)
                        .on_press(move |_| filter.set(f))
                        .child(f.label())
                })),
        )
        .child(
            ScrollView::new().height(Size::flex(1.)).children(
                tasks
                    .read()
                    .iter()
                    .filter(|t| selected.matches(t))
                    // Key by stable id, never by index: rows reorder and delete.
                    .map(|task| TaskRow::new(task.clone(), tasks).key(task.id)),
            ),
        )
}

#[derive(PartialEq)]
struct TaskRow {
    task: Task,
    tasks: State<Vec<Task>>,
}

impl TaskRow {
    fn new(task: Task, tasks: State<Vec<Task>>) -> Self {
        Self { task, tasks }
    }
}

impl Component for TaskRow {
    fn render(&self) -> impl IntoElement {
        let id = self.task.id;
        let mut tasks = self.tasks;
        let done = self.task.done;

        rect()
            .horizontal()
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .spacing(8.)
            .padding(8.)
            .child(
                label()
                    .text(self.task.title.clone())
                    .width(Size::flex(1.))
                    .font_size(15.),
            )
            .child(
                Button::new()
                    .on_press(move |_| {
                        if let Some(task) = tasks.write().iter_mut().find(|t| t.id == id) {
                            task.done = !task.done;
                        }
                    })
                    .child(if done { "Undo" } else { "Done" }),
            )
            .child(
                Button::new()
                    .on_press(move |_| tasks.write().retain(|t| t.id != id))
                    .child("Delete"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::prelude::*;

    #[test]
    fn renders() {
        let mut runner = launch_test(app);
        runner.sync_and_update();
        assert!(
            runner
                .find(|_, element| {
                    Label::try_downcast(element)
                        .filter(|label| label.text == "My tasks")
                        .is_some()
                })
                .is_some()
        );
    }
}

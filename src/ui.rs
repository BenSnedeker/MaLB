

pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    Burts,
    Log,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> Self {
        match input {
            MenuItem::Home => 0,
            MenuItem::Burts => 1,
            MenuItem::Log => 2,
        }
    }
}


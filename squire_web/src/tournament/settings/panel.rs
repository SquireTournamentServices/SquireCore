use std::{fmt::Display, marker::PhantomData, rc::Rc, str::FromStr};

use squire_sdk::model::settings::{PairingSetting, TournamentSetting};
use yew::prelude::*;

use crate::utils::TextInput;

use super::SettingsMessage;

pub struct SettingPanel {
    label: &'static str,
    convert: Rc<dyn Fn(String) -> Option<TournamentSetting>>,
    emitter: Callback<TournamentSetting>,
}

pub fn make_panel<T, G, S>(
    emitter: &Callback<TournamentSetting>,
    label: &'static str,
    item: G,
) -> SettingPanel
where
    T: 'static + FromStr,
    G: 'static + Copy + Fn(T) -> S,
    S: 'static + Into<TournamentSetting>,
{
    SettingPanel::new(label, item, emitter.clone())
}


fn make_chain<T, F, S>(f: F) -> impl Clone + Fn(String) -> Option<TournamentSetting>
where
    T: 'static + FromStr,
    F: 'static + Copy + Fn(T) -> S,
    S: 'static + Into<TournamentSetting>,
{
    move |s: String| s.parse().ok().map(f).map(Into::into)
}

impl SettingPanel {
    pub fn new<T, G, S>(
        label: &'static str,
        convert: G,
        emitter: Callback<TournamentSetting>,
    ) -> SettingPanel
    where
        T: 'static + FromStr,
        G: 'static + Copy + Fn(T) -> S,
        S: 'static + Into<TournamentSetting>,
    {
        SettingPanel {
            label,
            convert: Rc::new(make_chain(convert)),
            emitter,
        }
    }

    #[allow(clippy::option_map_unit_fn)]
    pub fn view<T: Display>(&self, data: T) -> Html {
        let convert = self.convert.clone();
        let emitter = self.emitter.clone();
        let process = move |s| {
            convert(s).map(|out| emitter.emit(out));
        };
        html! {
            <div>
                <p>{ format!("{} {data}", self.label) }</p>
                <TextInput label = { " change to " } process = { process }/>
            </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use squire_sdk::model::settings::PairingSetting;

    use crate::tournament::settings::panel::make_chain;

    #[test]
    fn test_converter() {
        let f = make_chain(PairingSetting::MatchSize);
        let opt = f(Default::default());
        assert!(opt.is_none());
        let opt = f(String::from("abc"));
        assert!(opt.is_none());
        let opt = f(String::from("5"));
        assert!(opt.is_some());
        println!("{opt:?}");
    }
}

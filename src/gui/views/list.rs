use crate::gui::style;
use crate::core::uad_lists::{ UadLists, PackageState, Package };
use std::{collections::HashMap};

use iced::{
    scrollable, Align, Column, Command, Container, Element, Space,
    Length, Row, Scrollable, Text, text_input, TextInput, Svg,
    PickList, pick_list, Button, button, HorizontalAlignment, VerticalAlignment
};

use crate::core::sync::{ 
    list_all_system_packages, hashset_installed_system_packages, 
    uninstall_package, restore_package,
};

#[derive(Default, Debug, Clone)]
pub struct List {
    ready: bool,
    p_row: Vec<PackageRow>,
    phone_packages_row: Vec<PackageRow>,
    packages: String,
    input: text_input::State,
    description: String,
    package_scrollable_state: scrollable::State,
    package_state_picklist: pick_list::State<PackageState>,
    list_picklist: pick_list::State<UadLists>,
    selected_package_state: Option<PackageState>,
    selected_list: Option<UadLists>,
    pub input_value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchInputChanged(String),
    LoadPackages(&'static HashMap<String, Package>),
    ListSelected(UadLists),
    PackageStateSelected(PackageState),
    List(usize, RowMessage),
    NoEvent,
}


impl List {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackages(uad_lists) => {
                self.packages = list_all_system_packages();
                self.p_row = Vec::new();
                self.selected_package_state = Some(PackageState::Installed);
                self.selected_list = Some(UadLists::All);

                let installed_system_packages = hashset_installed_system_packages();
                let mut description;
                let mut uad_list;
                let mut state;

                for p_name in self.packages.lines() {
                    state = "installed";
                    description = "[No description]";
                    uad_list = "unlisted";
                    
                    if uad_lists.contains_key(p_name) {
                        description = uad_lists.get(p_name).unwrap().description.as_ref().unwrap();
                        uad_list = &uad_lists.get(p_name).unwrap().list;
                    }

                    if !installed_system_packages.contains(p_name) {
                        state = "uninstalled";
                    }

                    let package_row = PackageRow::new(
                        &p_name,
                        &state,
                        &description,
                        &uad_list,
                    );
                    self.p_row.push(package_row)
                }
                self.p_row.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                self.phone_packages_row = self.p_row.clone();
                Self::filter_package_lists(self);
                self.ready = true;
                Command::none()
            }
            Message::SearchInputChanged(letter) => {
                self.input_value = letter;
                Self::filter_package_lists(self);
                Command::none()
            }
            Message::ListSelected(list) => {
                self.selected_list = Some(list);
                Self::filter_package_lists(self);
                Command::none()
            }

            Message::PackageStateSelected(package_state) => {
                self.selected_package_state = Some(package_state);
                Self::filter_package_lists(self);
                Command::none()
            },
            Message::List(i, row_message) => {
                let package = self.p_row[i].clone();
                self.description = package.description;
                self.p_row[i].update(row_message).map(move |row_message| Message::List(i, row_message))
            }
            Message::NoEvent => Command::none(),
        }


    }
    pub fn view(&mut self) -> Element<Message> {
        if self.ready {
            let search_packages = TextInput::new(
                &mut self.input,
                "Search packages...",
                &mut self.input_value,
                Message::SearchInputChanged,
            )
            .padding(5);

            // let package_amount = Text::new(format!("{} packages found", packages.len()));

            let divider = Space::new(Length::Fill, Length::Shrink);

            let list_picklist = PickList::new(
                        &mut self.list_picklist,
                        &UadLists::ALL[..],
                        self.selected_list,
                        Message::ListSelected,
                    );

            let package_state_picklist = PickList::new(
                        &mut self.package_state_picklist,
                        &PackageState::ALL[..],
                        self.selected_package_state,
                        Message::PackageStateSelected,
                    );

            let control_panel = Row::new()
                .width(Length::Fill)
                .align_items(Align::Center)
                .spacing(10)
                .push(search_packages)
                .push(divider)
                .push(package_state_picklist)
                .push(list_picklist);

            let package_name = Text::new("Package").width(Length::FillPortion(6));
            let package_state = Text::new("State").width(Length::FillPortion(3));

            let package_panel = Row::new()
                .width(Length::Fill)
                .align_items(Align::Center)
                .padding(5)
                .push(package_name)
                .push(package_state);
                
            // let mut packages_v: Vec<&str> = self.packages.lines().collect();
            let description_panel = Row::new()
                .width(Length::Fill)
                .align_items(Align::Center)
                .height(Length::FillPortion(2))
                .push(Text::new(&self.description));

            let test = self.p_row
                .iter_mut()
                .enumerate()
                .fold(Column::new().spacing(6), |col, (i, p)| {
                    col.push(p.view().map(move |msg| Message::List(i, msg)))
                });

            let packages_scrollable = Scrollable::new(&mut self.package_scrollable_state)
                .push(test)
                .spacing(5)
                .align_items(Align::Center)
                .height(Length::FillPortion(6))
                .style(style::Scrollable);

            let content = Column::new()
                .width(Length::Fill)
                .spacing(10)
                .align_items(Align::Center)
                .push(control_panel)
                .push(package_panel)
                .push(packages_scrollable)
                .push(description_panel);

            Container::new(content)
                .height(Length::Fill)
                .padding(10)
                .style(style::Content)
                .into()
        } else {
            loading_data()
        }
    }

    fn filter_package_lists(&mut self) {

        let list_filter: UadLists = self.selected_list.unwrap();
        let package_filter: PackageState = self.selected_package_state.unwrap();

        let mut filtered_packages: Vec<PackageRow> = self.phone_packages_row
            .iter()
            .filter(
                |p|
                (p.name.contains(&self.input_value) || self.input_value.is_empty()) && 
                (p.state == package_filter.to_string() || package_filter == PackageState::All) &&
                (p.uad_list.to_string() == list_filter.to_string() || list_filter == UadLists::All)
            )
            .cloned()
            .collect();
            
        filtered_packages.sort_by(|a, b| a.name.cmp(&b.name));
        self.p_row = filtered_packages;
    }
}

#[derive(Clone, Debug)]
pub struct PackageRow {
    pub name: String,
    pub state: String,
    pub description: String,
    pub uad_list: String,
    package_btn_state: button::State,
    remove_restore_btn_state: button::State,
}

#[derive(Clone, Debug)]
pub enum RowMessage {
    NoEvent,
    RemovePressed(PackageRow),
    RestorePressed(PackageRow),
}

impl PackageRow {
    pub fn new(
        name: &str,
        state: &str,
        description: &str,
        uad_list: &str,

    ) -> Self {
        Self {
            name: name.to_string(),
            state: state.to_string(),
            description: description.to_string(),
            uad_list: uad_list.to_string(),
            remove_restore_btn_state: button::State::default(),
            package_btn_state: button::State::default(),
        }
    }

    pub fn update(&mut self, message: RowMessage) -> Command<RowMessage> {
        match message {
            RowMessage::RemovePressed(package) => {
                uninstall_package(package.name);
                self.state = "uninstalled".to_string();
                Command::none()
            }
            RowMessage::RestorePressed(package) => {
                restore_package(package.name);
                self.state = "installed".to_string();
                Command::none()
            }
            RowMessage::NoEvent => Command::none(),
        }
    }

    pub fn view(&mut self) -> Element<RowMessage> {
        let package = self.clone();
        let add_svg_path = format!("{}/ressources/assets/trash.svg", env!("CARGO_MANIFEST_DIR"));

        let content = Button::new(
            &mut self.package_btn_state,
            Row::new()
                .align_items(Align::Center)
                .push(Text::new(&self.name).width(Length::FillPortion(6)))
                .push(Text::new(&self.state).width(Length::FillPortion(3)))
                .push(if self.state == "installed" {
                                            Button::new(
                                                &mut self.remove_restore_btn_state,
                                                Svg::from_path(add_svg_path)
                                                    .width(Length::Fill)
                                                    .height(Length::Fill),
                                            )
                                            .on_press(RowMessage::RemovePressed(package))
                                            .style(style::PrimaryButton::Enabled)
                                        } else {
                                            Button::new(
                                                &mut self.remove_restore_btn_state,
                                                Text::new("Restore")
                                                    .width(Length::Fill)
                                                    .horizontal_alignment(HorizontalAlignment::Center),
                                            )
                                            .on_press(RowMessage::RestorePressed(package))
                                            .style(style::PrimaryButton::Enabled)
                                        })
            )
            .style(style::PackageRow::Enabled)
            .on_press(RowMessage::NoEvent);

        Column::new().push(content).into()


    }
}

fn loading_data<'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("Pulling packages from the phone. Please wait...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .size(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .center_x()
    .style(style::Content)
    .into()
}
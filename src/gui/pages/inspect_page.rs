use iced::alignment::Horizontal;
use iced::widget::{
    Button, Column, Container, PickList, Row, Scrollable, Text, TextInput, Tooltip,
};
use iced::{alignment, Alignment, Font, Length};
use iced_lazy::lazy;
use iced_native::widget::scrollable::Properties;
use iced_native::widget::tooltip::Position;
use iced_native::widget::{button, horizontal_space, Rule};

use crate::gui::components::tab::get_pages_tabs;
use crate::gui::components::types::my_modal::MyModal;
use crate::gui::styles::style_constants::{get_font, ICONS, SARASA_MONO_SC_BOLD};
use crate::gui::styles::types::element_type::ElementType;
use crate::gui::styles::types::style_tuple::StyleTuple;
use crate::gui::types::message::Message;
use crate::networking::types::search_parameters::SearchParameters;
use crate::report::get_report_entries::get_searched_entries;
use crate::translations::translations_2::{
    country_translation, showing_results_translation, sort_by_translation,
};
use crate::utils::formatted_strings::{get_connection_color, get_open_report_tooltip};
use crate::{Language, ReportSortType, RunningPage, Sniffer, StyleType};

/// Computes the body of gui inspect page
pub fn inspect_page(sniffer: &Sniffer) -> Container<Message> {
    let font = get_font(sniffer.style);

    let mut body = Column::new()
        .width(Length::Fill)
        .padding(10)
        .spacing(10)
        .align_items(Alignment::Center);

    let mut tab_and_body = Column::new().height(Length::Fill);

    let tabs = get_pages_tabs(
        [
            RunningPage::Overview,
            RunningPage::Inspect,
            RunningPage::Notifications,
        ],
        &["d ", "5 ", "7 "],
        &[
            Message::ChangeRunningPage(RunningPage::Overview),
            Message::TickInit,
            Message::ChangeRunningPage(RunningPage::Notifications),
        ],
        RunningPage::Inspect,
        sniffer.style,
        sniffer.language,
        sniffer.unread_notifications,
    );

    tab_and_body = tab_and_body.push(tabs);

    tab_and_body = tab_and_body.push(filters_row(
        sniffer.search.clone(),
        sniffer.style,
        sniffer.language,
    ));

    let sort_active_str = sniffer
        .report_sort_type
        .get_picklist_label(sniffer.language);
    let sort_list_str: Vec<String> = ReportSortType::all_strings(sniffer.language);
    let picklist_sort = PickList::new(
        sort_list_str.clone(),
        Some(sort_active_str),
        move |selected_str| {
            if selected_str == *sort_list_str.get(0).unwrap_or(&String::new()) {
                Message::ReportSortSelection(ReportSortType::MostRecent)
            } else if selected_str == *sort_list_str.get(1).unwrap_or(&String::new()) {
                Message::ReportSortSelection(ReportSortType::MostBytes)
            } else {
                Message::ReportSortSelection(ReportSortType::MostPackets)
            }
        },
    )
    .padding([3, 7])
    .font(font)
    .style(StyleTuple(sniffer.style, ElementType::Standard));

    let report = lazy(
        (
            sniffer.runtime_data.tot_sent_packets + sniffer.runtime_data.tot_received_packets,
            sniffer.style,
            sniffer.language,
            sniffer.report_sort_type,
            sniffer.search.clone(),
            sniffer.page_number,
        ),
        move |_| lazy_report(sniffer),
    );

    body = body
        .push(
            Row::new()
                .align_items(Alignment::Center)
                .spacing(10)
                .push(Text::new(sort_by_translation(sniffer.language)).font(font))
                .push(picklist_sort),
        )
        .push(report);

    Container::new(Column::new().push(tab_and_body.push(body)))
        .height(Length::Fill)
        .style(<StyleTuple as Into<iced::theme::Container>>::into(
            StyleTuple(sniffer.style, ElementType::Standard),
        ))
}

fn lazy_report(sniffer: &Sniffer) -> Column<'static, Message> {
    let font = get_font(sniffer.style);

    let (search_results, results_number) = get_searched_entries(sniffer);

    let mut col_report = Column::new().height(Length::Fill).width(Length::Fill);
    col_report = col_report
        .push(Text::new("       Src IP address       Src port      Dst IP address       Dst port  Layer4   Layer7     Packets      Bytes   Country").font(font))
        .push(Rule::horizontal(20).style(<StyleTuple as Into<iced::theme::Rule>>::into(StyleTuple(
            sniffer.style,
            ElementType::Standard,
        ))))
    ;
    let mut scroll_report = Column::new();
    let start_entry_num = (sniffer.page_number - 1) * 10 + 1;
    let end_entry_num = start_entry_num + search_results.len() - 1;
    for (key, val, flag) in search_results {
        let entry_color = get_connection_color(val.traffic_direction, sniffer.style);
        let entry_row = Row::new()
            .align_items(Alignment::Center)
            .push(
                Text::new(format!("  {}{}", key.print_gui(), val.print_gui()))
                    .style(iced::theme::Text::Color(entry_color))
                    .font(SARASA_MONO_SC_BOLD),
            )
            .push(flag)
            .push(Text::new("  "));

        scroll_report = scroll_report.push(
            button(entry_row)
                .padding(2)
                .on_press(Message::ShowModal(MyModal::ConnectionDetails(val.index)))
                .style(StyleTuple(sniffer.style, ElementType::Neutral).into()),
        );
    }
    col_report = col_report.push(Container::new(
        Scrollable::new(scroll_report)
            .horizontal_scroll(Properties::new())
            .style(<StyleTuple as Into<iced::theme::Scrollable>>::into(
                StyleTuple(sniffer.style, ElementType::Standard),
            )),
    ));

    Column::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(
            Row::new()
                .spacing(15)
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .push(horizontal_space(Length::FillPortion(1)))
                .push(
                    Container::new(col_report)
                        .padding([10, 7, 7, 7])
                        .height(Length::Fixed(310.0))
                        .width(Length::Fixed(1050.0))
                        .style(<StyleTuple as Into<iced::theme::Container>>::into(
                            StyleTuple(sniffer.style, ElementType::BorderedRound),
                        )),
                )
                .push(
                    Container::new(get_button_open_report(
                        sniffer.style,
                        sniffer.language,
                        font,
                    ))
                    .width(Length::FillPortion(1)),
                ),
        )
        .push(
            Row::new()
                .align_items(Alignment::Center)
                .spacing(10)
                .push(if sniffer.page_number > 1 {
                    Container::new(get_button_change_page(sniffer.style, false).width(30.0))
                } else {
                    Container::new(horizontal_space(30.0))
                })
                .push(
                    Text::new(showing_results_translation(
                        sniffer.language,
                        start_entry_num,
                        end_entry_num,
                        results_number,
                    ))
                    .font(font),
                )
                .push(
                    if sniffer.page_number < f32::ceil(results_number as f32 / 10.0) as usize {
                        Container::new(get_button_change_page(sniffer.style, true).width(30.0))
                    } else {
                        Container::new(horizontal_space(30.0))
                    },
                ),
        )
}

fn filters_row(
    search_params: SearchParameters,
    style: StyleType,
    language: Language,
) -> Row<'static, Message> {
    let font = get_font(style);

    let mut row_filters = Row::new().spacing(10).padding(15);

    row_filters = row_filters.push(country_filter(search_params, font, style, language));

    row_filters
}

fn country_filter(
    search_params: SearchParameters,
    font: Font,
    style: StyleType,
    language: Language,
) -> Container<'static, Message> {
    let is_filter_active = !search_params.country.is_empty();

    let button_clear = button(
        Text::new("x")
            .font(font)
            .horizontal_alignment(Horizontal::Center)
            .size(15),
    )
    .padding(2)
    .height(Length::Fixed(20.0))
    .width(Length::Fixed(20.0))
    .style(StyleTuple(style, ElementType::Standard).into())
    .on_press(Message::Search(SearchParameters {
        country: String::new(),
        ..search_params.clone()
    }));

    let input = TextInput::new("+", &search_params.country)
        .on_input(move |new_country| {
            Message::Search(SearchParameters {
                country: new_country.trim().to_string(),
                ..search_params.clone()
            })
        })
        .padding([0, 5])
        .font(font)
        .width(Length::Fixed(if is_filter_active { 30.0 } else { 20.0 }))
        .style(<StyleTuple as Into<iced::theme::TextInput>>::into(
            StyleTuple(
                style,
                if is_filter_active {
                    ElementType::Badge
                } else {
                    ElementType::Standard
                },
            ),
        ));

    let mut content = Row::new()
        .spacing(5)
        .push(Text::new(format!("{}:", country_translation(language))).font(font))
        .push(input);

    if is_filter_active {
        content = content.push(button_clear);
    }

    Container::new(content)
        .padding(15)
        .style(<StyleTuple as Into<iced::theme::Container>>::into(
            StyleTuple(
                style,
                if is_filter_active {
                    ElementType::Badge
                } else {
                    ElementType::Standard
                },
            ),
        ))
}

fn get_button_change_page(style: StyleType, increment: bool) -> Button<'static, Message> {
    button(
        Text::new(if increment { "j" } else { "i" })
            .size(12.0)
            .font(ICONS)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .padding(5)
    .height(Length::Fixed(30.0))
    .width(Length::Fixed(30.0))
    .style(StyleTuple(style, ElementType::Standard).into())
    .on_press(Message::UpdatePageNumber(increment))
}

fn get_button_open_report(
    style: StyleType,
    language: Language,
    font: Font,
) -> Tooltip<'static, Message> {
    let content = button(
        Text::new('8'.to_string())
            .font(ICONS)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .padding(10)
    .height(Length::Fixed(50.0))
    .width(Length::Fixed(75.0))
    .style(StyleTuple(style, ElementType::Standard).into())
    .on_press(Message::OpenReport);

    Tooltip::new(content, get_open_report_tooltip(language), Position::Top)
        .gap(5)
        .font(font)
        .style(<StyleTuple as Into<iced::theme::Container>>::into(
            StyleTuple(style, ElementType::Tooltip),
        ))
}

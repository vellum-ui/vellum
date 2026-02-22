use masonry::properties::types::CrossAxisAlignment;
use masonry::widgets::Flex;

/// Create the initial widget tree for the application.
/// This is an empty root Flex column tagged with ROOT_FLEX_TAG.
/// It fills the entire window, letting JS-created children control their own layout.
pub fn create_initial_ui() -> Flex {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .must_fill_main_axis(true)
}

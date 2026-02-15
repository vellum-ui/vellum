use masonry::properties::types::CrossAxisAlignment;
use masonry::widgets::Flex;

/// Create the initial widget tree for the application.
/// This is an empty root Flex column tagged with ROOT_FLEX_TAG.
/// All JS-created widgets will be dynamically added as children of this container.
pub fn create_initial_ui() -> Flex {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true)
}

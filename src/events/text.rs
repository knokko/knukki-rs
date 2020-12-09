/// This event is for the `on_char_type` method of `Component`. 
/// 
/// This event indicates that the user typed a single character (actually a 
/// grapheme cluster), but the application didn't explicitly prompt the user
/// to type something using `request_text_input`.
/// 
/// ### Limitations
/// Note that this event can only be fired if the user has some kind of
/// keyboard. If no keyboard is available, only the `request_text_input` method
/// of the component buddy can be used to ask the user for text input.
pub struct CharTypeEvent {

    text: String
}
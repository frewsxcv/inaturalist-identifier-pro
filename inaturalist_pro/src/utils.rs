/// Utility functions for image URL manipulation
use std::borrow::Cow;

/// Converts iNaturalist square image URLs to original resolution URLs
///
/// This function replaces "/square.jpg" with "/original.jpg" in iNaturalist image URLs.
/// If the URL doesn't contain "/square.jpg", it returns the original URL unchanged.
///
/// # Examples
///
/// ```
/// use inaturalist_pro::utils::to_original_image_url;
///
/// let square_url = "https://static.inaturalist.org/photos/117822320/square.jpg";
/// let original_url = to_original_image_url(square_url);
/// assert_eq!(original_url, "https://static.inaturalist.org/photos/117822320/original.jpg");
///
/// let other_url = "https://example.com/image.png";
/// let unchanged_url = to_original_image_url(other_url);
/// assert_eq!(unchanged_url, "https://example.com/image.png");
/// ```
pub fn to_original_image_url(url: &str) -> Cow<'_, str> {
    if url.contains("square.jpg") {
        Cow::Owned(url.replace("square.jpg", "original.jpg"))
    } else {
        Cow::Borrowed(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_original_image_url_with_square() {
        let input = "https://static.inaturalist.org/photos/117822320/square.jpg";
        let expected = "https://static.inaturalist.org/photos/117822320/original.jpg";
        assert_eq!(to_original_image_url(input), expected);
    }

    #[test]
    fn test_to_original_image_url_without_square() {
        let input = "https://example.com/image.png";
        assert_eq!(to_original_image_url(input), input);
    }

    #[test]
    fn test_to_original_image_url_with_different_extension() {
        let input = "https://static.inaturalist.org/photos/117822320/square.png";
        assert_eq!(to_original_image_url(input), input);
    }

    #[test]
    fn test_to_original_image_url_multiple_occurrences() {
        let input = "https://static.inaturalist.org/photos/117822320/square.jpg?square.jpg";
        let expected = "https://static.inaturalist.org/photos/117822320/original.jpg?original.jpg";
        assert_eq!(to_original_image_url(input), expected);
    }

    #[test]
    fn test_to_original_image_url_empty_string() {
        let input = "";
        assert_eq!(to_original_image_url(input), input);
    }
}

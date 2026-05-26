#![allow(dead_code)]
// Embedded content and media elements
tag_funcs!(video, audio, picture, canvas, iframe);
tag_funcs_void!(img, source, track, embed);

pub fn canvas_sized(width: u32, height: u32) -> Box<crate::view_components::leafs::ViewLeafText> {
    canvas("")
        .attr("width", &width.to_string())
        .attr("height", &height.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_components::Brick;

    fn rh(c: &dyn Brick) -> String {
        crate::view_components::render_to_html(c)
    }

    #[test]
    fn img_is_void() {
        assert_eq!(rh(&*img()), "<img>");
    }

    #[test]
    fn img_with_src_alt() {
        assert_eq!(
            rh(&*img().attr("src", "/logo.png").attr("alt", "Logo")),
            r#"<img src="/logo.png" alt="Logo">"#
        );
    }

    #[test]
    fn video_renders() {
        assert_eq!(
            rh(&*video("").c("player")),
            r#"<video class="player"></video>"#
        );
    }

    #[test]
    fn source_is_void() {
        assert_eq!(
            rh(&*source().attr("src", "/clip.mp4").attr("type", "video/mp4")),
            r#"<source src="/clip.mp4" type="video/mp4">"#
        );
    }

    #[test]
    fn canvas_with_dimensions() {
        assert_eq!(
            rh(&*canvas("").attr("width", "800").attr("height", "600")),
            r#"<canvas width="800" height="600"></canvas>"#
        );
    }

    #[test]
    fn canvas_sized_sets_width_and_height() {
        assert_eq!(
            rh(&*canvas_sized(800, 600)),
            r#"<canvas width="800" height="600"></canvas>"#
        );
    }
}

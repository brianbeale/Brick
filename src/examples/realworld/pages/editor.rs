use super::super::api::{self, CreateArticleRequest, UpdateArticleRequest};
use super::super::auth::AuthStore;
use super::super::routes::{RwRoute, navigate};
use crate::view_components::*;

#[model]
pub struct EditorPage {
    #[prop]
    pub slug: String,
    #[validate(required)]
    pub title: String,
    #[validate(required)]
    pub description: String,
    #[validate(required)]
    pub body: String,
    #[default(String::new())]
    pub tag_input: String,
    pub tags: List<String>,
    #[default(String::new())]
    pub error: String,
    #[default(false)]
    pub has_error: bool,
    #[default(false)]
    pub saving: bool,
}

#[controller]
impl EditorPage {
    #[on(mount)]
    pub fn init(&mut self) {
        let slug = self.slug.clone();
        if slug.is_empty() {
            return;
        }
        let title = self.title.clone();
        let description = self.description.clone();
        let body = self.body.clone();
        let tags = std::rc::Rc::clone(&self.tags);
        #[cfg(brick_dom)]
        spawn_local(async move {
            if let Ok(art) = api::get_article(&slug).await {
                title.set(art.title);
                description.set(art.description);
                body.set(art.body);
                let mut list = tags.borrow_mut();
                for tag in art.tag_list {
                    list.push(tag);
                }
            }
        });
    }

    pub fn add_tag(&mut self) {
        let tag = self.tag_input.read();
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            return;
        }
        self.tags.borrow_mut().push(tag);
        self.tag_input.set(String::new());
    }

    #[on(submit)]
    pub fn save(&mut self) {
        let auth = AuthStore::get();
        let token = auth.token.read();
        if token.is_empty() {
            self.error
                .set("You must be logged in to publish.".to_string());
            return;
        }
        let title = self.title.read();
        let description = self.description.read();
        let body_text = self.body.read();
        let tag_list: Vec<String> = self
            .tags
            .borrow()
            .items()
            .iter()
            .map(|t| (**t).clone())
            .collect();
        let slug = self.slug.clone();
        let error = self.error.clone();
        let has_error = self.has_error.clone();
        let saving = self.saving.clone();
        let token2 = token.clone();
        saving.set(true);
        #[cfg(brick_dom)]
        spawn_local(async move {
            let result = if slug.is_empty() {
                let req = CreateArticleRequest {
                    title,
                    description,
                    body: body_text,
                    tag_list,
                };
                api::create_article(&token2, req).await
            } else {
                let req = UpdateArticleRequest {
                    title: Some(title),
                    description: Some(description),
                    body: Some(body_text),
                };
                api::update_article(&token2, &slug, req).await
            };
            saving.set(false);
            match result {
                Ok(art) => navigate(RwRoute::article(art.slug)),
                Err(e) => {
                    error.set(e);
                    has_error.set(true);
                }
            }
        });
    }
}

#[view(EditorPage)]
fn render() -> Box<ViewComposite> {
    style! {
        .editor-page { max-width: 720px; margin: 2rem auto; padding: 0 1rem; }
        .field { display: flex; flex-direction: column; gap: 0.25rem; margin-bottom: 1rem; }
        .field input, .field textarea { margin-bottom: 0; font-size: 1rem; }
        .field textarea { min-height: 10rem; resize: vertical; }
        .tag-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
        .tag-row input { flex: 1; }
        .tag-list { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-bottom: 1rem; }
        .tag-item { padding: 0.2rem 0.5rem; background: #333; color: white; border-radius: 3px; font-size: 0.85rem; }
        .error-msg { color: #e53e3e; margin-bottom: 1rem; }
        .error { color: #e53e3e; font-size: 0.8rem; min-height: 1.2rem; }
    }
    children! {
        div {
            class("editor-page"),
            h1("Editor"),
            when!(my.saving,
                p("Saving…").c("error-msg"),
            ),
            when!(my.has_error,
                p(live!("{my.error}")).c("error-msg"),
            ),
            div {
                class("field"),
                input().attr("type", "text").attr("placeholder", "Article Title").bind(&my.title),
                when!(my.title_touched,
                    p(live!("{my.title_error}")).c("error"),
                ),
            },
            div {
                class("field"),
                input().attr("type", "text").attr("placeholder", "Short description").bind(&my.description),
                when!(my.description_touched,
                    p(live!("{my.description_error}")).c("error"),
                ),
            },
            div {
                class("field"),
                input().attr("type", "text").attr("placeholder", "Write your article (markdown)").bind(&my.body),
                when!(my.body_touched,
                    p(live!("{my.body_error}")).c("error"),
                ),
            },
            div {
                class("tag-row"),
                input().attr("type", "text").attr("placeholder", "Enter tags").bind(&my.tag_input),
                button("Add Tag").trigger(&my.add_tag),
            },
            div {
                class("tag-list"),
                list!(my.tags, |tag| span(tag.as_str()).c("tag-item")),
            },
            when!(my.valid,
                button("Publish Article").primary().submit(&my.save),
                button("Publish Article").primary().attr("disabled", "true"),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mgmt::cascade;

    fn page() -> EditorPage {
        EditorPage {
            slug: String::new(),
            ..cascade()
        }
    }

    #[test]
    fn renders_editor_heading() {
        assert!(crate::view_components::render_to_html(&*page().into_component()).contains("Editor"));
    }

    #[test]
    fn initial_not_valid() {
        assert!(!page().valid.read());
    }

    #[test]
    fn add_tag_pushes_to_list() {
        let mut p = page();
        p.tag_input.set("rust".to_string());
        p.add_tag();
        let list = p.tags.borrow();
        let items: Vec<_> = list.items().iter().map(|t| (**t).clone()).collect();
        assert!(items.contains(&"rust".to_string()));
    }

    #[test]
    fn add_tag_clears_input() {
        let mut p = page();
        p.tag_input.set("rust".to_string());
        p.add_tag();
        assert!(p.tag_input.read().is_empty());
    }

    #[test]
    fn add_empty_tag_ignored() {
        let mut p = page();
        p.tag_input.set("   ".to_string());
        p.add_tag();
        assert_eq!(p.tags.borrow().items().len(), 0);
    }
}

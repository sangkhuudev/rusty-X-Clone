use indexmap::IndexMap;
use uchat_domain::PostId;
use uchat_endpoint::post::types::PublicPost;

#[derive(Default)]
pub struct PostManager {
    pub posts: IndexMap<PostId, PublicPost>
}

impl PostManager {
    pub fn update<F>(&mut self, post_id: PostId, mut update_fn: F) -> bool 
    where
        F: FnMut(&mut PublicPost)
    {
        if let Some(post) = self.posts.get_mut(&post_id) {
            update_fn(post);
            true
        } else {
            false
        }
    }

    pub fn populate<T>(&mut self, posts: T)
    where
        T: Iterator<Item = PublicPost>
    {
        self.posts.clear();
        for post in posts {
            self.posts.insert(post.id, post);
        }
    }

    pub fn clear(&mut self) {
        self.posts.clear()
    }

    pub fn get(&self, post_id: &PostId) -> Option<&PublicPost> {
        self.posts.get(post_id)
    }

    pub fn remove(&mut self, post_id: &PostId) {
        self.posts.shift_remove(post_id);
    }
}    
use async_graphql::connection::{query, Connection, Edge, EmptyFields};
use async_graphql::{Context, Result};
use pxid::Pxid;

use townhall::shared::pagination::Pagination;

use crate::context::SharedContext;
use crate::graphql::connection_details::ConnectionDetails;
use crate::graphql::modules::post::types::Post;

pub type PostsConnection = Connection<Pxid, Post, ConnectionDetails, EmptyFields>;

pub struct Posts;

impl Posts {
    pub async fn exec(
        ctx: &Context<'_>,
        after: Option<Pxid>,
        before: Option<Pxid>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<PostsConnection> {
        let context = ctx.data_unchecked::<SharedContext>();
        let after = after.map(|a| a.to_string());
        let before = before.map(|a| a.to_string());

        query(
            after,
            before,
            first,
            last,
            |after: Option<Pxid>,
             before: Option<Pxid>,
             first: Option<usize>,
             last: Option<usize>| async move {
                let pagination = Pagination::new(after, before, first, last)?;
                let query_set = context.services.post.list(Some(pagination), None).await?;
                let total_count = query_set.count();
                let posts = query_set.records();
                let page_info = pagination.get_page_info(total_count);
                let mut connection = PostsConnection::with_additional_fields(
                    page_info.has_prev_pages,
                    page_info.has_next_pages,
                    ConnectionDetails { total_count },
                );

                connection.edges.extend(posts.into_iter().filter_map(|p| {
                    match Post::try_from(p) {
                        Ok(p) => Some(Edge::new(p.id, p)),
                        Err(err) => {
                            tracing::error!(%err, "Failed to create post instance from result");
                            None
                        }
                    }
                }));

                Ok::<PostsConnection, async_graphql::Error>(connection)
            },
        )
        .await
    }
}

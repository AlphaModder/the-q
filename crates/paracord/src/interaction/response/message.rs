use std::{
    borrow::{Borrow, BorrowMut},
    convert::Infallible,
};

use qcore::builder;
use serenity::{
    builder::{
        CreateInteractionResponseData, CreateInteractionResponseFollowup, EditInteractionResponse,
    },
    model::{
        id::{RoleId, UserId},
        prelude::AttachmentType,
    },
    utils::MessageBuilder,
};

use super::{Components, Embed, Embeds, MessageComponent, ResponseData};

/// The body of a message
#[derive(Debug, qcore::Borrow)]
pub struct MessageBody<I, E> {
    content: MessageBuilder,
    embeds: Embeds,
    ping_replied: bool,
    ping_users: Vec<UserId>,
    ping_roles: Vec<RoleId>,
    #[borrow(mut)]
    components: Components<I, MessageComponent, E>,
}

macro_rules! build_body {
    ($self:expr, $builder:expr, $fn:ident) => {{
        let Self {
            content,
            embeds,
            ping_replied,
            ping_users,
            ping_roles,
            components,
        } = $self;
        components.$fn(embeds.$fn($builder.content(content)).allowed_mentions(|m| {
            m.replied_user(ping_replied)
                .users(ping_users)
                .roles(ping_roles)
        }))
    }};
}

impl<I, E> MessageBody<I, E> {
    /// Construct a new rich-text message using the given closure
    #[inline]
    pub fn rich(f: impl FnOnce(&mut MessageBuilder) -> &mut MessageBuilder) -> Self {
        let mut content = MessageBuilder::new();
        f(&mut content);
        Self {
            content,
            embeds: Embeds::default(),
            ping_replied: false,
            ping_users: vec![],
            ping_roles: vec![],
            components: Components::default(),
        }
    }

    /// Construct a new plaintext message
    #[inline]
    pub fn plain(c: impl Into<serenity::utils::Content>) -> Self {
        Self::rich(|mb| mb.push_safe(c))
    }

    /// Purge any validation errors caused during initialization
    ///
    /// # Errors
    /// If any component on the message contains an error it will be returned.
    #[inline]
    pub fn prepare(self) -> Result<MessageBody<I, Infallible>, E> {
        let Self {
            content,
            embeds,
            ping_replied,
            ping_users,
            ping_roles,
            components,
        } = self;
        Ok(MessageBody {
            content,
            embeds,
            ping_replied,
            ping_users,
            ping_roles,
            components: components.prepare()?,
        })
    }
}

impl<I> MessageBody<I, Infallible> {
    /// Apply the values of this message body to a response edit builder
    #[inline]
    pub fn build_edit_response(
        self,
        res: &mut EditInteractionResponse,
    ) -> &mut EditInteractionResponse {
        build_body!(self, res, build_edit_response)
    }

    /// Apply the values of this message body to a followup message builder
    #[inline]
    pub fn build_followup<'a, 'b>(
        self,
        fup: &'b mut CreateInteractionResponseFollowup<'a>,
    ) -> &'b mut CreateInteractionResponseFollowup<'a> {
        build_body!(self, fup, build_followup)
    }
}

#[builder(trait_name = MessageBodyExt)]
/// Helper methods for mutating [`MessageBody`]
impl<I, E> MessageBody<I, E> {
    /// Set whether the replied-to user is allowed to be pinged
    pub fn ping_replied(&mut self, ping_replied: bool) { self.ping_replied = ping_replied; }

    /// Set which users are allowed to be pinged
    pub fn ping_users(&mut self, ping_users: Vec<UserId>) { self.ping_users = ping_users; }

    /// Set which guild roles are allowed to be pinged
    pub fn ping_roles(&mut self, ping_roles: Vec<RoleId>) { self.ping_roles = ping_roles; }

    /// Add an embed to this message
    pub fn embed(&mut self, embed: Embed) { self.embeds.0.push(embed); }

    /// Add an embed to this message using the given closure
    pub fn build_embed(&mut self, f: impl FnOnce(Embed) -> Embed) {
        self.embed(f(Embed::default()));
    }
}

impl<'a, I> ResponseData<'a> for MessageBody<I, Infallible> {
    #[inline]
    fn build_response_data<'b>(
        self,
        data: &'b mut CreateInteractionResponseData<'a>,
    ) -> &'b mut CreateInteractionResponseData<'a> {
        build_body!(self, data, build_response_data)
    }
}

/// Options to provide when creating (or deferring the creation of) a message
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageOpts {
    tts: bool,
    ephemeral: bool,
}

macro_rules! build_opts {
    ($self:expr, $builder:expr) => {{
        let Self { tts, ephemeral } = $self;
        $builder.tts(tts).ephemeral(ephemeral)
    }};
}

impl MessageOpts {
    #[inline]
    fn build_followup<'a, 'b>(
        self,
        fup: &'a mut CreateInteractionResponseFollowup<'b>,
    ) -> &'a mut CreateInteractionResponseFollowup<'b> {
        build_opts!(self, fup)
    }
}

#[builder(trait_name = MessageOptsExt)]
/// Helper methods for mutating [`MessageOpts`]
impl MessageOpts {
    /// Set whether this message should be read by screen readers
    pub fn tts(&mut self, tts: bool) { self.tts = tts; }

    /// Set whether this message should be a private temporary response
    pub fn ephemeral(&mut self, ephemeral: bool) { self.ephemeral = ephemeral; }
}

impl<'a> ResponseData<'a> for MessageOpts {
    #[inline]
    fn build_response_data<'b>(
        self,
        data: &'b mut CreateInteractionResponseData<'a>,
    ) -> &'b mut CreateInteractionResponseData<'a> {
        build_opts!(self, data)
    }
}

/// A message
#[derive(Debug, qcore::Borrow)]
pub struct Message<'a, I, E> {
    #[borrow(mut)]
    body: MessageBody<I, E>,
    #[borrow(mut)]
    opts: MessageOpts,
    attachments: Vec<AttachmentType<'a>>,
}

impl<'a, I, E> Borrow<Components<I, MessageComponent, E>> for Message<'a, I, E> {
    fn borrow(&self) -> &Components<I, MessageComponent, E> { &self.body.components }
}

impl<'a, I, E> BorrowMut<Components<I, MessageComponent, E>> for Message<'a, I, E> {
    fn borrow_mut(&mut self) -> &mut Components<I, MessageComponent, E> {
        &mut self.body.components
    }
}

macro_rules! build_msg {
    ($self:expr, $builder:expr, $fn:ident) => {{
        let Self {
            body,
            opts,
            attachments,
        } = $self;
        opts.$fn(body.$fn($builder)).files(attachments)
    }};
}

impl<'a, I, E> From<MessageBody<I, E>> for Message<'a, I, E> {
    fn from(body: MessageBody<I, E>) -> Self {
        Self {
            body,
            opts: MessageOpts::default(),
            attachments: vec![],
        }
    }
}

impl<'a, I, E> Message<'a, I, E> {
    /// Construct a new rich-text message using the given closure
    #[inline]
    pub fn rich(f: impl FnOnce(&mut MessageBuilder) -> &mut MessageBuilder) -> Self {
        MessageBody::rich(f).into()
    }

    /// Construct a new plaintext message
    #[inline]
    pub fn plain(c: impl Into<serenity::utils::Content>) -> Self { MessageBody::plain(c).into() }

    /// Construct a new message from its constituent parts
    #[inline]
    #[must_use]
    pub fn from_parts(
        body: MessageBody<I, E>,
        opts: MessageOpts,
        attachments: Vec<AttachmentType<'a>>,
    ) -> Self {
        Self {
            body,
            opts,
            attachments,
        }
    }

    /// Purge any validation errors caused during initialization
    ///
    /// # Errors
    /// If the message body contains an error it will be returned.
    #[inline]
    pub fn prepare(self) -> Result<Message<'a, I, Infallible>, E> {
        let Self {
            body,
            opts,
            attachments,
        } = self;
        Ok(Message {
            body: body.prepare()?,
            opts,
            attachments,
        })
    }
}

impl<'a, I> Message<'a, I, Infallible> {
    /// Apply the values of this message to a followup message builder
    pub fn build_followup<'b>(
        self,
        fup: &'b mut CreateInteractionResponseFollowup<'a>,
    ) -> &'b mut CreateInteractionResponseFollowup<'a> {
        build_msg!(self, fup, build_followup)
    }
}

#[builder(trait_name = MessageExt)]
/// Helper methods for mutating [`Message`]
impl<'a, I, E> Message<'a, I, E> {
    /// Add an attachment to this message
    pub fn attach(&mut self, attachments: impl IntoIterator<Item = AttachmentType<'a>>) {
        self.attachments.extend(attachments);
    }
}

impl<'a, I> ResponseData<'a> for Message<'a, I, Infallible> {
    #[inline]
    fn build_response_data<'b>(
        self,
        data: &'b mut CreateInteractionResponseData<'a>,
    ) -> &'b mut CreateInteractionResponseData<'a> {
        build_msg!(self, data, build_response_data)
    }
}

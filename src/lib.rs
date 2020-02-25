#![cfg_attr(not(feature = "std"), no_std)]

/// For more guidance on FRAME pallets, see the example.
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

pub mod defaults;

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, Parameter};
use sp_runtime::traits::{Member, SimpleArithmetic};
use system::ensure_signed;
use pallet_timestamp;

use defaults::*;
use serde::export::{Into, From, Default, Option};

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Change<T: Trait> {
  pub account: T::AccountId,
  pub block: T::BlockNumber,
  pub time: T::Moment,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Blog<T: Trait> {
  pub id: T::BlogId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,

  // Can be updated by the owner:
  pub writers: Vec<T::AccountId>,
  pub slug: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub posts_count: u16,
  pub followers_count: u32,

  pub edit_history: Vec<BlogHistoryRecord<T>>,

  pub score: i32,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct BlogUpdate<T: Trait> {
  pub writers: Option<Vec<T::AccountId>>,
  pub slug: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct BlogHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: BlogUpdate<T>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Post<T: Trait> {
  pub id: T::PostId,
  pub blog_id: T::BlogId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,
  pub extension: PostExtension<T>,

  // Next fields can be updated by the owner only:

  pub ipfs_hash: Vec<u8>,

  pub comments_count: u16,
  pub upvotes_count: u16,
  pub downvotes_count: u16,
  pub shares_count: u16,

  pub edit_history: Vec<PostHistoryRecord<T>>,

  pub score: i32,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PostUpdate<T: Trait> {
  pub blog_id: Option<T::BlogId>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PostHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: PostUpdate<T>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub enum PostExtension<T: Trait> {
  RegularPost,
  SharedPost(T::PostId),
  SharedComment(T::CommentId),
}

impl<T: Trait> Default for PostExtension<T> {
  fn default() -> Self {
    PostExtension::RegularPost
  }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Comment<T: Trait> {
  pub id: T::CommentId,
  pub parent_id: Option<T::CommentId>,
  pub post_id: T::PostId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,

  // Can be updated by the owner:
  pub ipfs_hash: Vec<u8>,

  pub upvotes_count: u16,
  pub downvotes_count: u16,
  pub shares_count: u16,
  pub direct_replies_count: u16,

  pub edit_history: Vec<CommentHistoryRecord<T>>,

  pub score: i32,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct CommentUpdate {
  pub ipfs_hash: Vec<u8>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct CommentHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: CommentUpdate,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub enum ReactionKind {
  Upvote,
  Downvote,
}

impl Default for ReactionKind {
  fn default() -> Self {
    ReactionKind::Upvote
  }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Reaction<T: Trait> {
  pub id: T::ReactionId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,
  pub kind: ReactionKind,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct SocialAccount<T: Trait> {
  pub followers_count: u32,
  pub following_accounts_count: u16,
  pub following_blogs_count: u16,
  pub reputation: u32,
  pub profile: Option<Profile<T>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Profile<T: Trait> {
  pub created: Change<T>,
  pub updated: Option<Change<T>>,

  pub username: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub edit_history: Vec<ProfileHistoryRecord<T>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ProfileUpdate {
  pub username: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ProfileHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: ProfileUpdate,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub enum ScoringAction {
  UpvotePost,
  DownvotePost,
  SharePost,
  CreateComment,
  UpvoteComment,
  DownvoteComment,
  ShareComment,
  FollowBlog,
  FollowAccount,
}

impl Default for ScoringAction {
  fn default() -> Self {
    ScoringAction::FollowAccount
  }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
  type BlogId: Parameter + Member + SimpleArithmetic + Default + Copy
  + From<usize> + From<u64>;

  type PostId: Parameter + Member + SimpleArithmetic + Default + Copy
  + From<usize> + From<u64>;

  type CommentId: Parameter + Member + SimpleArithmetic + Default + Copy
  + From<usize> + From<u64>;

  type ReactionId: Parameter + Member + SimpleArithmetic + Default + Copy
  + From<usize> + From<u64>;

  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    pub SlugMinLen get(slug_min_len): u32 = DEFAULT_SLUG_MIN_LEN;
    pub SlugMaxLen get(slug_max_len): u32 = DEFAULT_SLUG_MAX_LEN;

    pub IpfsHashLen get(ipfs_hash_len): u32 = DEFAULT_IPFS_HASH_LEN;

    pub UsernameMinLen get(username_min_len): u32 = DEFAULT_USERNAME_MIN_LEN;
    pub UsernameMaxLen get(username_max_len): u32 = DEFAULT_USERNAME_MAX_LEN;

    pub BlogMaxLen get(blog_max_len): u32 = DEFAULT_BLOG_MAX_LEN;
    pub PostMaxLen get(post_max_len): u32 = DEFAULT_POST_MAX_LEN;
    pub CommentMaxLen get(comment_max_len): u32 = DEFAULT_COMMENT_MAX_LEN;

    pub UpvotePostActionWeight get (upvote_post_action_weight): i16 = DEFAULT_UPVOTE_POST_ACTION_WEIGHT;
    pub DownvotePostActionWeight get (downvote_post_action_weight): i16 = DEFAULT_DOWNVOTE_POST_ACTION_WEIGHT;
    pub SharePostActionWeight get (share_post_action_weight): i16 = DEFAULT_SHARE_POST_ACTION_WEIGHT;
    pub CreateCommentActionWeight get (create_comment_action_weight): i16 = DEFAULT_CREATE_COMMENT_ACTION_WEIGHT;
    pub UpvoteCommentActionWeight get (upvote_comment_action_weight): i16 = DEFAULT_UPVOTE_COMMENT_ACTION_WEIGHT;
    pub DownvoteCommentActionWeight get (downvote_comment_action_weight): i16 = DEFAULT_DOWNVOTE_COMMENT_ACTION_WEIGHT;
    pub ShareCommentActionWeight get (share_comment_action_weight): i16 = DEFAULT_SHARE_COMMENT_ACTION_WEIGHT;
    pub FollowBlogActionWeight get (follow_blog_action_weight): i16 = DEFAULT_FOLLOW_BLOG_ACTION_WEIGHT;
    pub FollowAccountActionWeight get (follow_account_action_weight): i16 = DEFAULT_FOLLOW_ACCOUNT_ACTION_WEIGHT;

    pub BlogById get(blog_by_id): map T::BlogId => Option<Blog<T>>;
    pub PostById get(post_by_id): map T::PostId => Option<Post<T>>;
    pub CommentById get(comment_by_id): map T::CommentId => Option<Comment<T>>;
    pub ReactionById get(reaction_by_id): map T::ReactionId => Option<Reaction<T>>;
    pub SocialAccountById get(social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;

    pub BlogIdsByOwner get(blog_ids_by_owner): map T::AccountId => Vec<T::BlogId>;
    pub PostIdsByBlogId get(post_ids_by_blog_id): map T::BlogId => Vec<T::PostId>;
    pub CommentIdsByPostId get(comment_ids_by_post_id): map T::PostId => Vec<T::CommentId>;

    pub ReactionIdsByPostId get(reaction_ids_by_post_id): map T::PostId => Vec<T::ReactionId>;
    pub ReactionIdsByCommentId get(reaction_ids_by_comment_id): map T::CommentId => Vec<T::ReactionId>;
    pub PostReactionIdByAccount get(post_reaction_id_by_account): map (T::AccountId, T::PostId) => T::ReactionId;
    pub CommentReactionIdByAccount get(comment_reaction_id_by_account): map (T::AccountId, T::CommentId) => T::ReactionId;

    pub BlogIdBySlug get(blog_id_by_slug): map Vec<u8> => Option<T::BlogId>;

    pub BlogsFollowedByAccount get(blogs_followed_by_account): map T::AccountId => Vec<T::BlogId>;
    pub BlogFollowers get(blog_followers): map T::BlogId => Vec<T::AccountId>;
    pub BlogFollowedByAccount get(blog_followed_by_account): map (T::AccountId, T::BlogId) => bool;

    pub AccountFollowedByAccount get(account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    pub AccountsFollowedByAccount get(accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    pub AccountFollowers get(account_followers): map T::AccountId => Vec<T::AccountId>;

    pub NextBlogId get(next_blog_id): T::BlogId = 1.into();
    pub NextPostId get(next_post_id): T::PostId = 1.into();
    pub NextCommentId get(next_comment_id): T::CommentId = 1.into();
    pub NextReactionId get(next_reaction_id): T::ReactionId = 1.into();

    pub AccountReputationDiffByAccount get(account_reputation_diff_by_account): map (T::AccountId, T::AccountId, ScoringAction) => Option<i16>; // TODO shorten name (?refactor)
    pub PostScoreByAccount get(post_score_by_account): map (T::AccountId, T::PostId, ScoringAction) => Option<i16>;
    pub CommentScoreByAccount get(comment_score_by_account): map (T::AccountId, T::CommentId, ScoringAction) => Option<i16>;

    pub PostSharesByAccount get(post_shares_by_account): map (T::AccountId, T::PostId) => u16;
    pub SharedPostIdsByOriginalPostId get(shared_post_ids_by_original_post_id): map T::PostId => Vec<T::PostId>;

    pub CommentSharesByAccount get(comment_shares_by_account): map (T::AccountId, T::CommentId) => u16;
    pub SharedPostIdsByOriginalCommentId get(shared_post_ids_by_original_comment_id): map T::CommentId => Vec<T::PostId>;

    pub AccountByProfileUsername get(account_by_profile_username): map Vec<u8> => Option<T::AccountId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your pallet
    fn deposit_event() = default;

    pub fn create_blog(origin, slug: Vec<u8>, ipfs_hash: Vec<u8>) {
      ensure_signed(origin)?;
    }

    // pub fn update_blog(origin, blog_id: T::BlogId, update: BlogUpdate<T>) {}

    pub fn follow_blog(origin, blog_id: T::BlogId) {}

    pub fn unfollow_blog(origin, blog_id: T::BlogId) {}

    pub fn follow_account(origin, account: T::AccountId) {}

    pub fn unfollow_account(origin, account: T::AccountId) {}

    pub fn create_profile(origin, username: Vec<u8>, ipfs_hash: Vec<u8>) {}

    pub fn update_profile(origin, update: ProfileUpdate) {}

    // pub fn create_post(origin, blog_id: T::BlogId, ipfs_hash: Vec<u8>, extension: PostExtension<T>) {}

    // pub fn update_post(origin, post_id: T::PostId, update: PostUpdate<T>) {}

    pub fn create_comment(origin, post_id: T::PostId, parent_id: Option<T::CommentId>, ipfs_hash: Vec<u8>) {}

    pub fn update_comment(origin, comment_id: T::CommentId, update: CommentUpdate) {}

    pub fn create_post_reaction(origin, post_id: T::PostId, kind: ReactionKind) {}

    pub fn update_post_reaction(origin, post_id: T::PostId, reaction_id: T::ReactionId, new_kind: ReactionKind) {}

    pub fn delete_post_reaction(origin, post_id: T::PostId, reaction_id: T::ReactionId) {}

    pub fn create_comment_reaction(origin, comment_id: T::CommentId, kind: ReactionKind) {}

    pub fn update_comment_reaction(origin, comment_id: T::CommentId, reaction_id: T::ReactionId, new_kind: ReactionKind) {}

    pub fn delete_comment_reaction(origin, comment_id: T::CommentId, reaction_id: T::ReactionId) {}
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
    <T as Trait>::BlogId,
    <T as Trait>::PostId,
    <T as Trait>::CommentId,
    <T as Trait>::ReactionId,
   {
    BlogCreated(AccountId, BlogId),
    BlogUpdated(AccountId, BlogId),
    BlogDeleted(AccountId, BlogId),

    BlogFollowed(AccountId, BlogId),
    BlogUnfollowed(AccountId, BlogId),

    AccountReputationChanged(AccountId, ScoringAction, u32),

    AccountFollowed(AccountId, AccountId),
    AccountUnfollowed(AccountId, AccountId),

    PostCreated(AccountId, PostId),
    PostUpdated(AccountId, PostId),
    PostDeleted(AccountId, PostId),
    PostShared(AccountId, PostId),

    CommentCreated(AccountId, CommentId),
    CommentUpdated(AccountId, CommentId),
    CommentDeleted(AccountId, CommentId),
    CommentShared(AccountId, CommentId),

    PostReactionCreated(AccountId, PostId, ReactionId),
    PostReactionUpdated(AccountId, PostId, ReactionId),
    PostReactionDeleted(AccountId, PostId, ReactionId),

    CommentReactionCreated(AccountId, CommentId, ReactionId),
    CommentReactionUpdated(AccountId, CommentId, ReactionId),
    CommentReactionDeleted(AccountId, CommentId, ReactionId),

    ProfileCreated(AccountId),
    ProfileUpdated(AccountId),
  }
);

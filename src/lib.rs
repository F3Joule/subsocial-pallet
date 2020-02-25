#![cfg_attr(not(feature = "std"), no_std)]

/// For more guidance on FRAME pallets, see the example.
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

pub mod defaults;
pub mod messages;

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, ensure};
use sp_runtime::{RuntimeDebug};
use system::ensure_signed;
use pallet_timestamp;

use defaults::*;
use messages::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Change<T: Trait> {
  pub account: T::AccountId,
  pub block: T::BlockNumber,
  pub time: T::Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Blog<T: Trait> {
  pub id: BlogId,
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BlogUpdate<T: Trait> {
  pub writers: Option<Vec<T::AccountId>>,
  pub slug: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BlogHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: BlogUpdate<T>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Post<T: Trait> {
  pub id: PostId,
  pub blog_id: BlogId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,
  pub extension: PostExtension,

  // Next fields can be updated by the owner only:

  pub ipfs_hash: Vec<u8>,

  pub comments_count: u16,
  pub upvotes_count: u16,
  pub downvotes_count: u16,
  pub shares_count: u16,

  pub edit_history: Vec<PostHistoryRecord<T>>,

  pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostUpdate {
  pub blog_id: Option<BlogId>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct PostHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: PostUpdate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum PostExtension {
  RegularPost,
  SharedPost(PostId),
  SharedComment(CommentId),
}

impl Default for PostExtension {
  fn default() -> Self {
    PostExtension::RegularPost
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Comment<T: Trait> {
  pub id: CommentId,
  pub parent_id: Option<CommentId>,
  pub post_id: PostId,
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CommentUpdate {
  pub ipfs_hash: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CommentHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: CommentUpdate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum ReactionKind {
  Upvote,
  Downvote,
}

impl Default for ReactionKind {
  fn default() -> Self {
    ReactionKind::Upvote
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Reaction<T: Trait> {
  pub id: ReactionId,
  pub created: Change<T>,
  pub updated: Option<Change<T>>,
  pub kind: ReactionKind,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SocialAccount<T: Trait> {
  pub followers_count: u32,
  pub following_accounts_count: u16,
  pub following_blogs_count: u16,
  pub reputation: u32,
  pub profile: Option<Profile<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Profile<T: Trait> {
  pub created: Change<T>,
  pub updated: Option<Change<T>>,

  pub username: Vec<u8>,
  pub ipfs_hash: Vec<u8>,

  pub edit_history: Vec<ProfileHistoryRecord<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileUpdate {
  pub username: Option<Vec<u8>>,
  pub ipfs_hash: Option<Vec<u8>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ProfileHistoryRecord<T: Trait> {
  pub edited: Change<T>,
  pub old_data: ProfileUpdate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
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

pub type BlogId = u64;
pub type PostId = u64;
pub type CommentId = u64;
pub type ReactionId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
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

    pub BlogById get(blog_by_id): map BlogId => Option<Blog<T>>;
    pub PostById get(post_by_id): map PostId => Option<Post<T>>;
    pub CommentById get(comment_by_id): map CommentId => Option<Comment<T>>;
    pub ReactionById get(reaction_by_id): map ReactionId => Option<Reaction<T>>;
    pub SocialAccountById get(social_account_by_id): map T::AccountId => Option<SocialAccount<T>>;

    pub BlogIdsByOwner get(blog_ids_by_owner): map T::AccountId => Vec<BlogId>;
    pub PostIdsByBlogId get(post_ids_by_blog_id): map BlogId => Vec<PostId>;
    pub CommentIdsByPostId get(comment_ids_by_post_id): map PostId => Vec<CommentId>;

    pub ReactionIdsByPostId get(reaction_ids_by_post_id): map PostId => Vec<ReactionId>;
    pub ReactionIdsByCommentId get(reaction_ids_by_comment_id): map CommentId => Vec<ReactionId>;
    pub PostReactionIdByAccount get(post_reaction_id_by_account): map (T::AccountId, PostId) => ReactionId;
    pub CommentReactionIdByAccount get(comment_reaction_id_by_account): map (T::AccountId, CommentId) => ReactionId;

    pub BlogIdBySlug get(blog_id_by_slug): map Vec<u8> => Option<BlogId>;

    pub BlogsFollowedByAccount get(blogs_followed_by_account): map T::AccountId => Vec<BlogId>;
    pub BlogFollowers get(blog_followers): map BlogId => Vec<T::AccountId>;
    pub BlogFollowedByAccount get(blog_followed_by_account): map (T::AccountId, BlogId) => bool;

    pub AccountFollowedByAccount get(account_followed_by_account): map (T::AccountId, T::AccountId) => bool;
    pub AccountsFollowedByAccount get(accounts_followed_by_account): map T::AccountId => Vec<T::AccountId>;
    pub AccountFollowers get(account_followers): map T::AccountId => Vec<T::AccountId>;

    pub NextBlogId get(next_blog_id): BlogId = 1;
    pub NextPostId get(next_post_id): PostId = 1;
    pub NextCommentId get(next_comment_id): CommentId = 1;
    pub NextReactionId get(next_reaction_id): ReactionId = 1;

    pub AccountReputationDiffByAccount get(account_reputation_diff_by_account): map (T::AccountId, T::AccountId, ScoringAction) => Option<i16>; // TODO shorten name (?refactor)
    pub PostScoreByAccount get(post_score_by_account): map (T::AccountId, PostId, ScoringAction) => Option<i16>;
    pub CommentScoreByAccount get(comment_score_by_account): map (T::AccountId, CommentId, ScoringAction) => Option<i16>;

    pub PostSharesByAccount get(post_shares_by_account): map (T::AccountId, PostId) => u16;
    pub SharedPostIdsByOriginalPostId get(shared_post_ids_by_original_post_id): map PostId => Vec<PostId>;

    pub CommentSharesByAccount get(comment_shares_by_account): map (T::AccountId, CommentId) => u16;
    pub SharedPostIdsByOriginalCommentId get(shared_post_ids_by_original_comment_id): map CommentId => Vec<PostId>;

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
      let owner = ensure_signed(origin)?;

      ensure!(slug.len() >= Self::slug_min_len() as usize, MSG_BLOG_SLUG_IS_TOO_SHORT);
      ensure!(slug.len() <= Self::slug_max_len() as usize, MSG_BLOG_SLUG_IS_TOO_LONG);
      ensure!(!BlogIdBySlug::exists(slug.clone()), MSG_BLOG_SLUG_IS_NOT_UNIQUE);
      // Self::is_ipfs_hash_valid(ipfs_hash.clone())?;

      let blog_id = Self::next_blog_id();
      let ref mut new_blog: Blog<T> = Blog {
        id: blog_id,
        created: Change {
          account: owner.clone(),
          block: <system::Module<T>>::block_number(),
          time: <pallet_timestamp::Module<T>>::now(),
        },
        updated: None,
        writers: vec![],
        slug: slug.clone(),
        ipfs_hash,
        posts_count: 0,
        followers_count: 0,
        edit_history: vec![],
        score: 0
      };

      // Blog creator automatically follows their blog:
      // Self::add_blog_follower_and_insert_blog(owner.clone(), new_blog, true)?;

      <BlogIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(blog_id));
      BlogIdBySlug::insert(slug, blog_id);
      NextBlogId::mutate(|n| { *n += 1; });
    }

    // pub fn update_blog(origin, blog_id: BlogId, update: BlogUpdate<T>) {}

    pub fn follow_blog(origin, blog_id: BlogId) {}

    pub fn unfollow_blog(origin, blog_id: BlogId) {}

    pub fn follow_account(origin, account: T::AccountId) {}

    pub fn unfollow_account(origin, account: T::AccountId) {}

    pub fn create_profile(origin, username: Vec<u8>, ipfs_hash: Vec<u8>) {}

    pub fn update_profile(origin, update: ProfileUpdate) {}

    pub fn create_post(origin, blog_id: BlogId, ipfs_hash: Vec<u8>, extension: PostExtension) {}

    // pub fn update_post(origin, post_id: PostId, update: PostUpdate<T>) {}

    pub fn create_comment(origin, post_id: PostId, parent_id: Option<CommentId>, ipfs_hash: Vec<u8>) {}

    pub fn update_comment(origin, comment_id: CommentId, update: CommentUpdate) {}

    pub fn create_post_reaction(origin, post_id: PostId, kind: ReactionKind) {}

    pub fn update_post_reaction(origin, post_id: PostId, reaction_id: ReactionId, new_kind: ReactionKind) {}

    pub fn delete_post_reaction(origin, post_id: PostId, reaction_id: ReactionId) {}

    pub fn create_comment_reaction(origin, comment_id: CommentId, kind: ReactionKind) {}

    pub fn update_comment_reaction(origin, comment_id: CommentId, reaction_id: ReactionId, new_kind: ReactionKind) {}

    pub fn delete_comment_reaction(origin, comment_id: CommentId, reaction_id: ReactionId) {}
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
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

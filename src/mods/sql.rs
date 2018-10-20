use diesel;
use diesel::prelude::*;

use uuid::Uuid;

use Models::{NewListing, Listing, NewTag, Tag, NewListingTag, ListingTag};

pub fn create_listing(conn: &PgConnection, checksum: &str, file_name: &str, file_path: &str, file_size: &i64) -> Listing {
    use Schema::listings;

    let new_listing = NewListing {
        id: &Uuid::new_v4().to_string(),
        checksum,
        file_name,
        file_path,
        file_size,
    };

    diesel::insert_into(listings::table)
        .values(&new_listing)
        .get_result(conn)
        .expect("Error saving new listing")
}

pub fn create_listing_tag(conn: &PgConnection, p_listing_id: &str, p_tag_id: &str) -> ListingTag {
    use Schema::listing_tags;
    use Schema::listing_tags::dsl::*;

    let new_listing_tag = NewListingTag {
        id: &Uuid::new_v4().to_string(),
        listing_id: &p_listing_id,
        tag_id: &p_tag_id
    };

    let single_listing_tag: Vec<ListingTag> = listing_tags
                                                .filter(listing_id.eq(p_listing_id))
                                                .filter(tag_id.eq(p_tag_id))
                                                .limit(1)
                                                .load::<ListingTag>(conn)
                                                .expect("Error loading listing tag");

    match single_listing_tag.len() {
        0 => {
                // TODO 18-10-13 Need proper error handling for when inserts fail eg foreign key violations.
                diesel::insert_into(listing_tags::table)
                    .values(&new_listing_tag)
                    .get_result(conn)
                    .expect("Error saving new listing tag")
            },
        _ => {
            let result: ListingTag = single_listing_tag.into_iter().nth(0).expect("Failed to load existing listing tag.");
            result
        }
    }
}

pub fn create_tag(conn: &PgConnection, p_tag: &str) -> Tag {
    use Schema::tags;
    use Schema::tags::dsl::*;

    let new_tag = NewTag {
        id: &Uuid::new_v4().to_string(),
        tag: p_tag
    };

    let single_tag: Vec<Tag> = tags
                                .filter(tag.eq(p_tag))
                                .limit(1)
                                .load::<Tag>(conn)
                                .expect("Error loading tag");

    match single_tag.len() {
        0 => {
            // TODO 18-10-13 Need proper error handling for when inserts fail eg foreign key violations.
            diesel::insert_into(tags::table)
                .values(&new_tag)
                .get_result(conn)
                .expect("Error saving new tag")
            },
        _ => {
            let result: Tag = single_tag.into_iter().nth(0).expect("Failed to load existing tag.");
            result
        }
    }
}

pub fn delete_listing(conn: &PgConnection, p_file_path: &str) {
    use Schema::listings::dsl::*;

    // TODO 18-09-23 One day, may want to mark listings as deleted instead of removing them
    diesel::delete(listings.filter(file_path.eq(p_file_path)))
        .execute(conn)
        .expect("Error deleting listing");
}

/// Deletes a listing tag while leaving associated tag untouched.
pub fn delete_listing_tag(conn: &PgConnection, p_listing_tag_id: &str) {
    use Schema::listing_tags::dsl::*;

    // TODO 18-10-20 One day, may want to mark listing_tags as deleted instead of removing them
    diesel::delete(listing_tags.filter(id.eq(p_listing_tag_id)))
        .execute(conn)
        .expect("Error deleting listing tag");
}

pub fn delete_tag(conn: &PgConnection, p_tag_id: &str) {
    // TODO 18-10-20 Feels hacky putting these deletes in blocks to avoid ambiguity on the id column... would like to see this nicer
    // TODO 18-10-20 One day, may want to mark listing_tags and tags as deleted instead of removing them
    {
        use Schema::listing_tags::dsl::*;
        diesel::delete(listing_tags.filter(tag_id.eq(p_tag_id)))
            .execute(conn)
            .expect("Error deleting listing tag");
    }
    {
        use Schema::tags::dsl::*;
        diesel::delete(tags.filter(id.eq(p_tag_id)))
            .execute(conn)
            .expect("Error deleting tag");
    }
}

pub fn establish_connection(connection: &str) -> PgConnection {
    PgConnection::establish(&connection).unwrap_or_else(|_| panic!("Error connecting to {}", connection))
}

pub fn find_single_file(conn: &PgConnection, p_file_path: &str) -> Vec<Listing> {
    use Schema::listings::dsl::*;
    listings
        .filter(file_path.eq(p_file_path))
        .limit(1)
        .load::<Listing>(conn)
        .expect("Error loading posts")
}

pub fn update_hash(conn: &PgConnection, id: &str, hash: &str) {
    use Schema::listings::dsl::{listings, checksum};
    diesel::update(listings.find(id))
        .set(checksum.eq(hash))
        .get_result::<Listing>(conn)
        .unwrap_or_else(|_| panic!("Unable to find post {}", id));
}
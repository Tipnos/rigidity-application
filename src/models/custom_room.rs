use crate::{schema::{custom_room_slots, custom_rooms, users}};
use crate::diesel::prelude::*;
use diesel::{PgConnection};
use serde::{Deserialize, Serialize};
use crate::enums::{Archetypes, GameModes, Maps};
use crate::models::{forms::custom_room::{CustomRoomForm, CustomRoomSlotForm}, ORMResult};
use std::{collections::HashMap};
use std::cmp::Eq;
use crate::handlers::custom_room::{CustomRoomData};
use diesel::result::Error;
use super::user::User;
use uuid::Uuid;
use rusoto_gamelift::{Player, StartMatchmakingInput, AttributeValue};

#[derive(Eq, Hash, Insertable, Identifiable, Serialize, Deserialize, Queryable, PartialEq)]
pub struct CustomRoom {
    pub id: i32,
    pub label: String,
    pub user_id: i32,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub current_game_mode: GameModes,
    pub current_map: Maps,
    pub matchmaking_ticket: Option<Uuid>,
}

impl CustomRoom {
    pub fn get_capacity(&self) -> usize {
        let capacity = (self.nb_teams * self.max_player_per_team) as usize;
        capacity
    }

    pub fn is_valid_slot(&self, team: &i32, team_position: &i32) -> bool {
        *team < self.nb_teams && *team_position < self.max_player_per_team
    }

    pub fn get_start_matchmaking_input(&self, tuples: &Vec<(CustomRoomSlot, User)>, ticket_id: &Uuid) -> StartMatchmakingInput {
        let mut players = Vec::new();

        for (slot, user) in tuples {
            if user.id == slot.user_id {
                let attributes = slot.get_gamelift_attributes(&user.nickname);

                players.push(Player {
                    latency_in_ms: None,
                    player_attributes: Some(attributes),
                    player_id: Some(slot.user_id.to_string()),
                    team: Some(slot.team.to_string()),
                });
            }
        }

        StartMatchmakingInput {
            configuration_name: self.current_map.to_string(),
            players: players,
            ticket_id: Some(ticket_id.to_string()),
        }
    }
}

#[derive(Insertable, Identifiable, Serialize, Deserialize, Queryable, Associations, PartialEq)]
#[belongs_to(CustomRoom, User)]
pub struct CustomRoomSlot {
    pub id: i32,
    pub custom_room_id: i32,
    pub team: i32,
    pub team_position: i32,
    pub user_id: i32,
    pub current_archetype: Archetypes,
}

impl CustomRoomSlot {
    pub fn get_gamelift_attributes(&self, nickname: &str) -> HashMap<String, AttributeValue> {
        let mut attributes = HashMap::new();
        attributes.insert(String::from("team"), AttributeValue { 
            s: None,
            n: Some(self.team as f64),
            sdm: None,
            sl: None
        });
        attributes.insert(String::from("team_position"), AttributeValue { 
            s: None,
            n: Some(self.team_position as f64),
            sdm: None,
            sl: None
        });
        attributes.insert(String::from("archetype"), AttributeValue { 
            s: None,
            n: Some(self.current_archetype.to_u32() as f64),
            sdm: None,
            sl: None
        });
        attributes.insert(String::from("nickname"), AttributeValue { 
            s: Some(nickname.to_owned()),
            n: None,
            sdm: None,
            sl: None
        });

        attributes
    }
}

pub fn get(id: &i32, conn: &PgConnection)
-> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    let custom_room = get_without_associations(id, conn)?;
    let slots = CustomRoomSlot::belonging_to(&custom_room)
        .load::<CustomRoomSlot>(conn)?;
    
    Ok((custom_room, slots))
}

pub fn get_with_users(id: &i32, conn: &PgConnection)
-> ORMResult<(CustomRoom, Vec<(CustomRoomSlot, User)>)> {
    use crate::schema::custom_room_slots::dsl::{custom_room_id as s_custom_room_id};

    let custom_room = get_without_associations(id, conn)?;
    let join = custom_room_slots::table.inner_join(users::table);
    let tuples = join.filter(s_custom_room_id.eq(id))
        .load::<(CustomRoomSlot, User)>(conn)?;

    Ok((custom_room, tuples))
}

pub fn get_without_associations(id: &i32, conn: &PgConnection)
-> ORMResult<CustomRoom> {
    use crate::schema::custom_rooms::dsl::{id as cr_id, custom_rooms};

    custom_rooms
        .filter(cr_id.eq(id))
        .get_result::<CustomRoom>(conn)
}

pub fn get_by_user_id(user_id: &i32, conn: &PgConnection)
-> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_rooms::dsl::{user_id as cr_user_id, custom_rooms};

    let custom_room = custom_rooms
        .filter(cr_user_id.eq(user_id))
        .get_result::<CustomRoom>(conn)?;

    let slots = CustomRoomSlot::belonging_to(&custom_room)
    .load::<CustomRoomSlot>(conn)?;
    
    Ok((custom_room, slots))
}

pub fn get_by_ticket_id(ticket_id: Uuid, conn: &PgConnection)
-> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_rooms::dsl::{matchmaking_ticket, custom_rooms};
    
    let custom_room = custom_rooms
        .filter(matchmaking_ticket.eq(ticket_id))
        .get_result::<CustomRoom>(conn)?;

    let slots = CustomRoomSlot::belonging_to(&custom_room)
    .load::<CustomRoomSlot>(conn)?;
    
    Ok((custom_room, slots))
}

pub fn get_all(conn: &PgConnection) 
-> ORMResult<HashMap<CustomRoom, Vec<CustomRoomSlot>>> {
    use crate::schema::custom_rooms::dsl::*;

    match custom_rooms.load::<CustomRoom>(conn) {
        Ok(c_rs) => {
            let mut result = HashMap::new();
            for c_r in c_rs {
                match CustomRoomSlot::belonging_to(&c_r)
                    .load::<CustomRoomSlot>(conn) {
                    Ok(slots) => {
                        result.insert(c_r, slots);
                    }
                    Err(err) => return Err(err)
                }
            }
            Ok(result)
        }
        Err(err) => return Err(err)
    }
}

pub fn get_slot_by_position(
    custom_room_id: &i32,
    team: &i32, 
    team_position: &i32, 
    conn: &PgConnection
) -> ORMResult<CustomRoomSlot> {
    use crate::schema::custom_room_slots::dsl::{
        team as s_team, 
        team_position as s_team_position, 
        custom_room_id as s_custom_room_id,
        custom_room_slots};

    custom_room_slots
        .filter(s_team.eq(team))
        .filter(s_team_position.eq(team_position))
        .filter(s_custom_room_id.eq(custom_room_id))
        .get_result::<CustomRoomSlot>(conn)
}

pub fn get_slot_by_user_id(
    user_id: &i32, 
    conn: &PgConnection
) -> ORMResult<CustomRoomSlot> {
    use crate::schema::custom_room_slots::dsl::{
        user_id as s_user_id, 
        custom_room_slots};

    custom_room_slots
        .filter(s_user_id.eq(user_id))
        .get_result::<CustomRoomSlot>(conn)
}

pub fn delete(
    user_id: &i32,
    conn: &PgConnection
) -> ORMResult<usize> {
    use crate::schema::custom_rooms::dsl::{user_id as u_id, custom_rooms};

    conn.transaction::<_, Error, _>(move || {    
        diesel::delete(custom_rooms.filter(u_id.eq(user_id)))
            .execute(conn)
    })
}

pub fn create(
    user_id: &i32, 
    data: CustomRoomData,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_rooms::dsl::{id, custom_rooms};
    use crate::schema::custom_room_slots::dsl::{custom_room_slots};

    conn.transaction::<(CustomRoom, Vec<CustomRoomSlot>), Error, _>(move || {
        diesel::insert_into(custom_rooms)
            .values(CustomRoomForm::new_from_data(
                &data, 
                user_id))
            .execute(conn)?;

        let custom_room_id = custom_rooms
            .select(id)
            .order(id.desc())
            .first(conn)?;

        diesel::insert_into(custom_room_slots)
            .values(CustomRoomSlotForm::new_from_custom_room_creation(
                &custom_room_id, 
                user_id))
            .execute(conn)?;

        get(&custom_room_id, conn)       
    })
}

pub fn update(
    user_id: &i32,
    custom_room_id: &i32, 
    data: &CustomRoomData,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_rooms::dsl::{id, custom_rooms};

    diesel::update(custom_rooms)
        .filter(id.eq(custom_room_id))
        .set(CustomRoomForm::new_from_data(
            data, 
            user_id)).execute(conn)?;

    get(&custom_room_id, conn)
}

pub fn update_ticket(
    custom_room_id: &i32,
    ticket_id: &Option<Uuid>,
    conn: &PgConnection
) -> ORMResult<()> {
    use crate::schema::custom_rooms::dsl::{matchmaking_ticket, id, custom_rooms};
    
    diesel::update(custom_rooms.filter(id.eq(custom_room_id)))
        .set(matchmaking_ticket.eq(ticket_id))
        .execute(conn)?;

    Ok(())
}

pub fn create_slot(
    custom_room_slot_form: &CustomRoomSlotForm,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_room_slots::dsl::{custom_room_slots};

    diesel::insert_into(custom_room_slots)
        .values(custom_room_slot_form)
        .execute(conn)?;

    get(&custom_room_slot_form.get_custom_room_id(), conn)  
} 

pub fn update_slot(
    user_id: &i32,
    custom_room_slot_form: &CustomRoomSlotForm,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_room_slots::dsl::{custom_room_id as s_custom_room_id, user_id as s_user_id, custom_room_slots}; 
    
    conn.transaction::<(CustomRoom, Vec<CustomRoomSlot>), Error, _>(move || {
        custom_room_slots
            .for_update()
            .filter(s_custom_room_id.eq(custom_room_slot_form.get_custom_room_id()))
            .load::<CustomRoomSlot>(conn)?;
        
        diesel::update(custom_room_slots.filter(s_user_id.eq(user_id)))
            .set(custom_room_slot_form)
            .execute(conn)?;

        get(&custom_room_slot_form.get_custom_room_id(), conn)  
    })
} 

pub fn update_slot_archetype(
    user_id: &i32,
    custom_room_id: &i32,
    archetype: &Archetypes,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_room_slots::dsl::{current_archetype, user_id as s_user_id, custom_room_slots};
    
    diesel::update(custom_room_slots.filter(s_user_id.eq(user_id)))
        .set(current_archetype.eq(archetype))
        .execute(conn)?;

    get(custom_room_id, conn)
}

pub fn delete_slot_by_user_id(
    custom_room_id: &i32,
    user_id: &i32,
    conn: &PgConnection
) -> ORMResult<(CustomRoom, Vec<CustomRoomSlot>)> {
    use crate::schema::custom_room_slots::dsl::{user_id as u_id, custom_room_slots};

    conn.transaction::<_, Error, _>(move || {    
        diesel::delete(custom_room_slots.filter(u_id.eq(user_id)))
            .execute(conn)
    })?;

    get(custom_room_id, conn)
}
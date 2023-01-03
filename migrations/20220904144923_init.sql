CREATE EXTENSION if not exists btree_gist;

create type meet_status_t as enum ('NEW', 'CANCELED', 'SCHEDULED', 'SPEC_WAITING', 'PAYMENT_WAITING');

create domain t_url as varchar(512);

create domain t_unsigned_int as integer
    constraint t_unsigned_int_check check (VALUE >= 0);

create domain t_unsigned_price as numeric(128, 64)
    constraint t_unsigned_price_check check (VALUE >= (0)::numeric);

create domain t_price as numeric(128, 64);

create domain t_positive_int as integer
    constraint t_positive_int_check check (VALUE > 0);

create domain t_positive_price as numeric(128, 64)
    constraint t_positive_price_check check (VALUE > (0)::numeric);

create table answers
(
    id   serial not null
        constraint answers_pk
            primary key,
    name text,
    type text
);

create table categories
(
    id        serial  not null
        constraint categories_pk
            primary key,
    parent_id integer not null,
    name_ru   text    not null,
    name_en   text,
    icon      text
);

create unique index categories_name_ru_uindex
    on categories (name_ru);

create table questions
(
    id     serial not null
        constraint questions_pk
            primary key,
    name   text,
    single boolean
);

create table services
(
    id               serial not null
        constraint services_pk
            primary key,
    name             text,
    parent_id        integer,
    name_alias       text,
    schema_id        integer,
    vjsf_id          integer,
    slug             text,
    last_answer_id   integer,
    icon             text,
    spec_title       text,
    spec_description text
);

create index services_last_answer_id_index
    on services (last_answer_id);

create table _questions_answers
(
    id          integer not null
        constraint tree3_pk
            primary key,
    parent_id   integer
        constraint tree3_tree3_id_fk
            references _questions_answers
            on update cascade on delete cascade,
    title       text not null,
    entity_type text,
    level       integer,
    slug        text,
    full_title  text,
    kind        text,
    spec_title  text
);

create index tree3_parent_id_index
    on _questions_answers (parent_id);

create table email_tokens
(
    token         bytea,
    email         text,
    device        text,
    expired_at    integer,
    id            bigserial                  not null
        constraint email_tokens_pk
            primary key,
    last_login_at integer,
    is_excluded   boolean default false not null,
    token_type    text                  not null
);

create table tokens
(
    token         bytea,
    account_id    integer,
    device        text,
    expired_at    integer,
    id            bigserial                  not null
        constraint tokens_pk
            primary key,
    last_login_at integer,
    is_excluded   boolean default false not null,
    token_type    text                  not null
);

create table spec
(
    first_name       text not null,
    last_name        text not null,
    middle_name      text,
    avatar_url       text,
    is_verified      boolean,
    passport_photo   bytea,
    id               integer not null
        constraint spec_profiles_pk
            primary key,
    avatar_thumb_url text,
    slug             varchar(64)
);

create unique index spec_profiles_slug_uindex
    on spec (slug);

create table spec_educations
(
    institution text not null,
    major       text not null,
    graduate    text,
    date_from   date not null,
    date_to     date,
    id          serial not null
        constraint service_educations_pk
            primary key,
    spec_id     integer
        constraint spec_services_educations_spec_id_fk
            references spec
);

create table calendar
(
    time_from bigint,
    time_to   bigint,
    range     tsrange,
    spec_id   integer
);

create index calendar_range_idx2
    on calendar (((date_part('epoch'::text, upper(range)) * 1000::double precision)::bigint));

create unique index calendar_spec_id_range_uindex
    on calendar (spec_id, range);

create table events
(
    id           serial not null
        constraint events_pk
            primary key,
    spec_id      integer,
    range        tsrange,
    user_message text,
    spec_message text,
    service_id   integer,
    meet_url     integer
);

create table meet_statuses
(
    id           text not null
        constraint meet_statuses_pk
            primary key,
    title_ru     text,
    user_can_see boolean,
    spec_can_see boolean
);

create table meet_status_flow
(
    status_id        text,
    parent_status_id text,
    role_id          text
);

create table main_services
(
    id integer not null
        constraint main_services_pk
            primary key
);

create table specs_services
(
    service_id integer not null
        constraint specs_services_main_services_id_fk
            references main_services
        constraint users_services_service__fk
            references services,
    price      integer,
    spec_id    integer not null,
    created_at timestamp default now(),
    constraint specs_services_pk
        primary key (spec_id, service_id)
);

create table spec_services_specializations
(
    updated_at        timestamp default now(),
    spec_id           integer not null
        constraint spec_services_specializations_spec_id_fk
            references spec,
    service_id        integer not null
        constraint spec_services_specializations_services_id_fk
            references services,
    specialization_id integer not null
        constraint spec_services_specializations_services_id_fk_2
            references services,
    constraint spec_services_specializations_pk
        primary key (spec_id, service_id, specialization_id),
    constraint spec_services_fk4
        foreign key (service_id, spec_id) references specs_services (service_id, spec_id)
);

create unique index main_services_id_uindex
    on main_services (id);

create table users
(
    id         serial not null
        constraint users_pk
            primary key,
    first_name varchar(32),
    last_name  varchar(32)
);

create table meets
(
    id                serial  not null
        constraint meets_pk
            primary key,
    range             tsrange not null,
    title             text,
    status_id         text    not null
        constraint meets_meet_statuses_id_fk
            references meet_statuses,
    price             integer not null,
    room              text,
    service_id        integer not null
        constraint meets_main_services_id_fk
            references main_services,
    spec_id           integer not null
        constraint meets_spec_id_fk
            references spec,
    user_id           integer
        constraint meets_users_id_fk
            references users,
    specialization_id integer,
    constraint meets_no_overlapping_time_ranges_spec2
        exclude using gist (spec_id with pg_catalog.=, range with pg_catalog.=),
    constraint meets_no_overlapping_time_ranges_spec
        exclude using gist (spec_id with pg_catalog.=, range with pg_catalog.=),
    constraint meets_no_overlapping_time_ranges_user
        exclude using gist (user_id with pg_catalog.=, range with pg_catalog.=),
    constraint meets_spec_services_specializations_fk1
        foreign key (spec_id, service_id, specialization_id) references spec_services_specializations
);

create table spec_reviews
(
    spec_id     integer                  not null
        constraint spec_reviews_spec_id_fk
            references spec,
    service_id  integer                  not null,
    content     varchar(2000),
    stars       integer        default 5 not null,
    created_at  timestamp      default now(),
    thumbs_up   t_unsigned_int default 0 not null,
    thumbs_down t_unsigned_int default 0 not null,
    user_id     integer                  not null
        constraint spec_reviews_users_id_fk
            references users,
    constraint spec_reviews_pk
        primary key (spec_id, user_id, service_id)
);

create table accounts
(
    email        text                               not null,
    password     bytea                              not null,
    id           bigserial                          not null
        constraint accounts_pk
            primary key,
    created_at   integer default date_part('epoch'::text, now()),
    confirmed_at integer,
    spec_id      integer
        constraint accounts_spec_id_fk
            references spec,
    user_id      integer
        constraint accounts_users_id_fk
            references users
);

create unique index accounts_email_uindex
    on accounts (email);

create unique index accounts_id_uindex
    on accounts (id);

create table spec_reviews_voters
(
    spec_id       integer not null,
    user_id       integer not null,
    service_id    integer not null,
    voter_user_id integer not null
        constraint spec_reviews_voters_users_id_fk
            references users,
    constraint spec_reviews_accounts_pk
        primary key (spec_id, user_id, service_id, voter_user_id),
    constraint spec_reviews_voters_spec_reviews_spec_id_service_id_user_id_fk
        foreign key (spec_id, service_id, user_id) references spec_reviews (spec_id, service_id, user_id),
    constraint spec_reviews_voters_check
        check (user_id <> voter_user_id)
);

create table notifications
(
    content       varchar(3000)           not null,
    email_sent_at timestamp,
    user_id       integer
        constraint notifications_users_id_fk
            references users,
    spec_id       integer
        constraint notifications_spec_id_fk
            references spec,
    viewed_at     timestamp,
    created_at    timestamp default now() not null,
    id            bigserial               not null
        constraint notifications_pk
            primary key
);


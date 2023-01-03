insert into public.users (id, first_name, last_name)
values  (1, 'Василий', 'Специалистович');

insert into public.users (id, first_name, last_name)
values  (2, 'Евгений', 'Клиентович');

insert into public.spec (first_name, last_name, middle_name, avatar_url, is_verified, passport_photo, id, avatar_thumb_url, slug)
values  ('Юрий', 'Фишер', 'Михайлович', 'images/e3782007-60ea-4819-8543-c2cf0ac58426/avatar/e1d1b994-19f1-4ea6-a203-2149bf33fe15.jpg', null, null, 1, 'images/e3782007-60ea-4819-8543-c2cf0ac58426/avatar/e1d1b994-19f1-4ea6-a203-2149bf33fe15_thumb.jpg', null);

insert into public.services (id, name, parent_id, name_alias, schema_id, vjsf_id, slug, last_answer_id, icon, spec_title, spec_description)
values  (6, 'Помощь при аренде помещений', 7, 'Юридическая консультация при аренде помещений', null, null, 'rent', null, null, null, null),
        (5, 'Помощь при покупке/продажи квартиры', 7, 'Юридическая консультация при покупке/продаже нежвижимости', null, null, 'trade', null, null, null, null),
        (2, 'Помощь в открытиии ИП', 1, 'Юридическая консультация по открытию ИП', null, null, 'ip', null, null, null, null),
        (3, 'Помощь в регистрации бизнеса', 1, 'Юридическая консультация по регистрации бизнеса', null, null, 'business', null, null, null, null),
        (7, 'Недвижимость', 1, 'Юридическая консультация при работе с недвижимостью', null, null, 'property', null, null, null, null),
        (1, 'Юридическая консультация', 0, 'Юридическая консультация онлайн', null, null, 'legal', null, 'law.svg', 'Юридический консультант', 'Станьте консультантом по юридическим вопросам'),
        (4, 'Психологическая помощь', 0, 'Психологическая помощь', null, null, 'psychological', null, 'psy.svg', 'Психолог', 'Оказывайте психологическую помощь'),
        (11, 'Зависимости', 4, 'Зависимости', null, null, 'addictions', null, null, null, null),
        (10, 'Наркотическая зависимость', 11, 'Психологическая помощь при наркотической зависимости', null, null, 'drugs', null, null, null, null),
        (9, 'Алкогольная зависимость', 11, 'Психологическая помощь при алкогольной зависимости', null, null, 'alco', null, null, null, null);

insert into public.meet_statuses (id, title_ru, user_can_see, spec_can_see)
values  ('New', 'Новая заявка', true, false),
        ('PaymentWaiting', 'Ожидание оплаты', true, true),
        ('Canceled', 'Отменена', true, true),
        ('Scheduled', 'Ожидание консультации', true, true),
        ('Finished', 'Консультация завершена', true, true),
        ('SpecWaiting', 'Ожидание подтвержния', true, true);

insert into public.meet_status_flow (status_id, parent_status_id, role_id)
values  ('Canceled', 'Scheduled', 'Spec'),
        ('Canceled', 'SpecWaiting', 'Spec'),
        ('Canceled', 'Scheduled', 'User'),
        ('Canceled', 'PaymentWaiting', 'User'),
        ('Canceled', 'PaymentWaiting', 'Spec'),
        ('Canceled', 'SpecWaiting', 'User'),
        ('Canceled', 'New', 'User'),
        ('PaymentWaiting', 'SpecWaiting', 'Spec');

insert into public.main_services (id)
values  (1),
        (4);

insert into public.specs_services (service_id, price, spec_id)
values  (4, 500, 1);

insert into public.spec_services_specializations (updated_at, spec_id, service_id, specialization_id)
values  ('2022-12-08 20:23:45.667255', 1, 4, 9);

insert into public.accounts (email, password, id, created_at, confirmed_at, spec_id, user_id)
values  ('kek@kek.kek', '0', 1, 1632412510, 1632412510, 1, null),
        ('2', '1', 2, 1632412510, 1632412510, null, 1);
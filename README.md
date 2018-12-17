# Laserlogin

Simple program to allow multiple users to use a laser cutter in conjunction with a script and a raspberry pi.
Most of the hard work is already done by the script.

## Setup
Install rust nightly from rustup.rs and run cargo build --release in the project directory.

Create a file called `first_user` and save your fh kiel email address into it.

Run the project via cargo run --release.

An administrative user is created and you should be able to log in on localhost:80.

Create further users via user settings.

Unlocking is done by GET-requesting /unlock/card-hash where card-hash is the users card hash.

Locking is done by GET-requesting /lock

A status page is shown at /status

## Maintenance

All data is kept in the file db.db in the project directory.

## Code structure

Laserlogin follows a basic model view controller structure.

All DB data structures are defined in the folder `model` and get initialized in `model/mod.rs`.

`mod.rs` also contains a few mutexed runtime variables for keeping login tokens and the current state of the laser.

User login works through three validation steps:

    1. Is the POST to /login well-formed HTTP?

    2. Can the users email address be found in the database?

    3. the `verify` method in `user.rs`

The verify method at current sends an HTTP-GET request with the credentials the user entered to the webdav server and looks for a `200 OK` status.
If the verify method returns `true`, a UUID-v4 token is generated and stored in an encrypted cookie on the user side, as well as in a hashmap in `mod.rs`.

User access gets restricted through Request Guards on methods. These guards basically try to construct a struct from request data, in this case they try to find the token 
contained in the request cookies in the hashmap in `mod.rs`. All of this is contained in `login_users.rs`.

The views are mostly handlebars templates. A base template (`base.hbs`) is filled with content through other templates.
If you want the menu bar to show up you need to pass a usertype to the template.

Status and Login are static HTML pages.

`main.rs` acts as request router.
Most GET requests are handled this way:

    1. Check guards to see if user may pass

    2. Request data from database

    3. Transform data (mostly into JsonValues, sometimes happens implicitly)

    4. Use transformed data to render a template


Most POST-requests are handled this way:

    1. Check guards

    2. Request object from db, see if collision will happen

    3. Update object in db

    4. Forward to appropriate view

Pretty much all responses are Result-Types which will automatically return error codes if a problem occurs and also allow for use of the ?-operator.

This project may break if the subcomponents update, I tested it a few days after the release of rocket.rs 0.4
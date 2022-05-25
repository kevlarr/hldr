# Placeholder

*Simple, declarative data seeding for PostgreSQL*

> See [Journey](https://github.com/kevlarr/jrny)
> for a complementary, straightforward SQL-based database migration tool.

**Important:** Placeholder is very, very alpha and feature-incomplete.

Placeholder strives to make generating _simple_ data much more succinct
and cleaner than using SQL, PL/pgSQL, or even other programming languages
with dedicated fixtures and factory libraries.

With some powerful, easy to read syntax and a single `hldr` command,
you can have a well populated database in no time without setting up
languages, dependencies, or verbose factory classes.

## Usage

Placeholder currently must be compiled from source but precompiled
binaries for common platforms should be [available soon](https://github.com/kevlarr/hldr/issues/16).

Once compiled and installed to your path,
to run the command you must specify the file to load and the
database connection string.

These can be specified by the `-f [or --data-file]` and
`-d [or --database-conn]` options.

```
# If installed in path as `hldr`
$ hldr -f path/to/data/file -d postgres://user:password@host:port/db
```

### Default files

There are several alternatives to using the command-line flags.

If the data file is not specified, `hldr` will by default look for a data file named `place.hldr`
in the current directory.

Additionally, a `.placehldr` file (also in the current directory) can be used to specify either
or both of the variables. For instance:

```
# For values with spaces, eg. the key-value connection string format, use double quotes.
# database_conn="host=localhost dbname=my_database user=postgres"
database_conn=postgres://postgres@localhost/my_database

# Specifying this means hldr will no longer look for the `place.hldr` default data file.
# The .hldr extension is recommended but not necessary.
data_file="my file.hldr"
```

Any variable in the `.placehldr` file will be overridden in favor of any command-line options
that are also supplied.

### Committing

By default, Placeholder will roll back the transaction,
which is useful to test that all records can be created
before finally applying them.
If you want to commit the records, pass the `--commit` flag.

```
$ hldr --commit
```

## Features

Placeholder uses a clean, whitespace-significant syntax,
with an indentation style of your choosing. Tabs or 3 spaces?
Do whatever you want, as long as it's consistent.

Records themselves can either be given a name, or they can be anonymous.
Naming records allows their columns (even those populated by the database)
to be referenced in other records.

There are literal values for booleans, numbers, and text strings.

- Numbers can be integers or floats; it will be up to the database to
coerce them to the right type based on the column.

- Text strings will also be coerced, which means they can be
used to represent `varchar`, `text`, `timestamptz`, arrays like `int[]`, or any
other type (even user-defined types) that can be constructed in SQL from string literals.

The general file format looks like...

```
schema
  table
    record
      column value
```
... where any number of schemas, tables, records, and attributes can be defined.

For example, a simple file that looks like...

```
public
  person
    fry
      name 'Philip J. Fry'
      hair_color 'red'

    leela
      name 'Turanga Leela'

  pet
    _
      name 'Nibbler'
```

... will create three record (two named, one anonymous) like:

```sql
INSERT INTO "public"."person" ("name", "hair_color")
  VALUES ('Philip J. Fry', 'red');

INSERT INTO "public"."person" ("name")
  VALUES ('Turanga Leela');

INSERT INTO "public"."pet" ("name")
  VALUES ('Nibbler');
```

Comments, like SQL, begin with `--` and can either be newline or trailing comments.

```
public
  -- This table has people in it
  person
    fry -- This is a named record
      name 'Philip J. Fry'

    -- This is an anonymous record...
    _
      -- ... even though we know its name
      name 'Morbo'
```

Bare identifiers (ie. `public`, `person`, and `name` in the example above)
are not lowercased or truncated automatically, like in SQL.
Statements use quoted identifiers automatically,
but (for the sake of the parser) you must explicitly quote identifiers
that have whitespace, punctuation, etc.

```
"schema with whitespace"
  "table.with -- dashes"
    my_record
      "column with spaces" 42
```

Records can also access values from other, named records using a fully-qualified
format like `schema.table@record_name.column`.
Columns do not need to be specified in the file to be referenced,
such as columns with database defaults like primary keys.

```
public
  person
    fry
      name 'Philip J. Fry'

  pet
    seymour
      name 'Seymour Asses'
      species 'Dog'
      person_id public.person@fry.id
```

Additionally, if the referenced table is in the same schema,
the schema name can be omitted.

```
public
  person
    fry
      name 'Philip J. Fry'

  pet
    seymour
      name 'Seymour Asses'
      species 'Dog'
      person_id person@fry.id
```

Likewise, if the referenced record is in the same table,
the table name can be omitted as well.

```
public
  bumblenum
    humble
      favorite_saying 'Yum yum!'

    stumble
      favorite_saying @humble.favorite_saying
```

Tables can also have aliases to have shorten qualified references.

```
public
  person as p
    p1
      name 'Person 1'

    p2
      name 'Person 2'

    p3
      name 'Person 3'

  pet
    _
      -- Aliases can be schema-qualified
      person_id public.p@fry.id

    _
      -- Or only be alias-qualified
      person_id p@fry.id

    _
      -- And the original table name can still be used
      person_id person@fry.id
```

## Planned features

See [enhancements](https://github.com/kevlarr/hldr/issues?q=is%3Aopen+is%3Aissue+label%3Aenhancement) for planned features.

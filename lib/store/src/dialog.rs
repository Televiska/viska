use crate::{
    db_conn,
    schema::{self, dialogs},
    DirtyTransaction, Error, Transaction,
};
use common::{
    chrono::{DateTime, Utc},
    libsip::core::method::Method,
    uuid::Uuid,
};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    prelude::*,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use std::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug},
    io::Write,
};

#[derive(Queryable, AsChangeset, Insertable, Debug, Clone)]
#[table_name = "dialogs"]
pub struct Dialog {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub computed_id: String,
    pub call_id: String,
    pub from_tag: String,
    pub to_tag: String,
    pub flow: DialogFlow,
}

#[derive(AsChangeset, Insertable, Debug, Default)]
#[table_name = "dialogs"]
pub struct DirtyDialog {
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub computed_id: Option<String>,
    pub call_id: Option<String>,
    pub from_tag: Option<String>,
    pub to_tag: Option<String>,
    pub flow: Option<DialogFlow>,
}

pub struct DirtyDialogWithTransaction {
    pub dialog: DirtyDialog,
    pub transaction: DirtyTransaction,
}

pub struct DialogWithTransaction {
    pub dialog: Dialog,
    pub transaction: Transaction,
}

impl From<(Dialog, Transaction)> for DialogWithTransaction {
    fn from(tuple: (Dialog, Transaction)) -> Self {
        Self {
            dialog: tuple.0,
            transaction: tuple.1,
        }
    }
}

pub struct LazyQuery {
    query: dialogs::BoxedQuery<'static, diesel::pg::Pg>,
}

impl LazyQuery {
    pub fn new(query: dialogs::BoxedQuery<'static, diesel::pg::Pg>) -> Self {
        Self { query }
    }

    pub fn computed_id(mut self, computed_id: Option<String>) -> Self {
        if let Some(computed_id) = computed_id {
            self.query = self.query.filter(dialogs::computed_id.eq(computed_id));
        }
        self
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(dialogs::created_at.desc());
        self
    }

    pub async fn load(self) -> Result<Vec<Dialog>, Error> {
        Ok(self.query.get_results(&db_conn()?)?)
    }

    pub async fn first(self) -> Result<Dialog, Error> {
        Ok(self.query.first(&db_conn()?)?)
    }
}

type DialogsWithTransactions = diesel::query_builder::BoxedSelectStatement<
    'static,
    (dialogs::SqlType, schema::transactions::SqlType),
    diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            schema::dialogs::table,
            schema::transactions::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<
            schema::dialogs::columns::id,
            schema::transactions::columns::dialog_id,
        >,
    >,
    diesel::pg::Pg,
>;
pub struct LazyQueryWithTransactions {
    query: DialogsWithTransactions,
}

impl LazyQueryWithTransactions {
    pub fn new(query: DialogsWithTransactions) -> Self {
        Self { query }
    }

    pub fn computed_id(mut self, computed_id: Option<String>) -> Self {
        if let Some(computed_id) = computed_id {
            self.query = self.query.filter(dialogs::computed_id.eq(computed_id));
        }
        self
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(dialogs::created_at.desc());
        self.query = self.query.order(schema::transactions::created_at.desc());
        self
    }

    pub fn load(self) -> Result<Vec<DialogWithTransaction>, Error> {
        Ok(self
            .query
            .get_results(&db_conn()?)?
            .into_iter()
            .map(|s: (Dialog, Transaction)| s.into())
            .collect())
    }

    pub fn first(self) -> Result<DialogWithTransaction, Error> {
        Ok(self
            .query
            .first::<(Dialog, Transaction)>(&db_conn()?)?
            .into())
    }
}

impl Dialog {
    pub fn query_with_transactions() -> LazyQueryWithTransactions {
        LazyQueryWithTransactions::new(
            dialogs::table
                .inner_join(
                    schema::transactions::table.on(dialogs::id.eq(schema::transactions::dialog_id)),
                )
                //.select(charge_locations::table::all_columns())
                .distinct()
                .into_boxed(),
        )
    }

    pub fn query() -> LazyQuery {
        LazyQuery::new(dialogs::table.into_boxed())
    }

    pub fn find(id: i64) -> Result<Self, Error> {
        Ok(dialogs::table.find(id).first::<Self>(&db_conn()?)?)
    }

    pub fn find_with_transaction(computed_id: String) -> Result<DialogWithTransaction, Error> {
        Ok(Self::query_with_transactions()
            .computed_id(Some(computed_id))
            .order_by_created_at()
            .first()?)
    }

    pub fn find_or_create_dialog(request: models::Request) -> Result<DialogWithTransaction, Error> {
        match request.dialog_id() {
            Some(dialog_id) => Ok(Self::find_with_transaction(dialog_id)?),
            None => Ok(Self::create_with_transaction(request)?),
        }
    }

    pub fn create(record: impl Into<DirtyDialog>) -> Result<Self, Error> {
        use diesel::insert_into;

        Ok(insert_into(dialogs::table)
            .values(record.into())
            .get_result(&db_conn()?)?)
    }

    pub fn create_with_transaction(
        dirty_struct: impl TryInto<DirtyDialogWithTransaction>,
    ) -> Result<DialogWithTransaction, Error> {
        use diesel::insert_into;

        let dirty_struct = dirty_struct.try_into().map_err(|_| {
            crate::Error::custom("Failed to convert dialog with transaction".into())
        })?;

        let connection = db_conn()?;
        Ok(connection.transaction::<_, Error, _>(|| {
            let dialog: Self = insert_into(dialogs::table)
                .values(dirty_struct.dialog)
                .get_result(&connection)?;

            let mut transaction = dirty_struct.transaction;
            transaction.dialog_id = Some(dialog.id);
            let transaction = insert_into(schema::transactions::table)
                .values(transaction)
                .get_result(&connection)?;

            Ok(DialogWithTransaction::from((dialog, transaction)))
        })?)
    }

    pub fn update(record: impl Into<DirtyDialog>, id: i64) -> Result<Self, Error> {
        Ok(diesel::update(dialogs::table.filter(dialogs::id.eq(id)))
            .set(&record.into())
            .get_result(&db_conn()?)?)
    }

    pub fn delete(id: i64) -> Result<Self, Error> {
        Ok(diesel::delete(dialogs::table.filter(dialogs::id.eq(id))).get_result(&db_conn()?)?)
    }
}

#[derive(FromSqlRow, AsExpression, Copy, Clone, PartialEq, Debug)]
#[sql_type = "Text"]
pub enum DialogFlow {
    Registration,
    Invite,
    Publish,
}
impl fmt::Display for DialogFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl ToSql<Text, Pg> for DialogFlow {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        <&str as ToSql<Text, Pg>>::to_sql(&self.to_string().to_lowercase().as_str(), out)
    }
}
impl FromSql<Text, Pg> for DialogFlow {
    fn from_sql(bytes: Option<diesel::pg::PgValue>) -> diesel::deserialize::Result<Self> {
        use std::str::FromStr;

        Ok(DialogFlow::from_str(
            <String as FromSql<Text, Pg>>::from_sql(bytes)?.as_ref(),
        )?)
    }
}

impl std::str::FromStr for DialogFlow {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case("registration") => Ok(DialogFlow::Registration),
            s if s.eq_ignore_ascii_case("invite") => Ok(DialogFlow::Invite),
            s if s.eq_ignore_ascii_case("publish") => Ok(DialogFlow::Publish),
            s => Err(format!("invalid DialogFlow `{}`", s)),
        }
    }
}

impl TryFrom<models::Request> for DirtyDialogWithTransaction {
    type Error = String;

    fn try_from(model: models::Request) -> Result<DirtyDialogWithTransaction, Self::Error> {
        //use store::{DialogFlow, RegistrationFlow};
        let call_id = match model.call_id() {
            Ok(call_id) => call_id,
            Err(_) => return Err("missing call id".into()),
        }
        .clone();

        let from_tag = match model.from_header_tag() {
            Ok(from_header_tag) => from_header_tag,
            Err(_) => return Err("missing from header tag".into()),
        }
        .clone();

        let branch_id = match model.via_header_branch() {
            Ok(branch_id) => branch_id,
            Err(_) => return Err("missing branch id".into()),
        }
        .clone();

        let to_tag = Uuid::new_v4();

        Ok(DirtyDialogWithTransaction {
            dialog: DirtyDialog {
                computed_id: Some(computed_id_for(&call_id, &from_tag, &to_tag)),
                call_id: Some(call_id),
                from_tag: Some(from_tag),
                to_tag: Some(to_tag.to_string()),
                flow: Some(flow_for_method(model.method().clone())?),
                ..Default::default()
            },
            transaction: crate::DirtyTransaction {
                branch_id: Some(branch_id),
                state: Some(crate::TransactionState::Trying),
                ..Default::default()
            },
        })
    }
}

impl Into<models::Dialog> for DialogWithTransaction {
    fn into(self) -> models::Dialog {
        models::Dialog {
            computed_id: self.dialog.computed_id,
            call_id: self.dialog.call_id,
            from_tag: self.dialog.from_tag,
            to_tag: self.dialog.to_tag,
            flow: into_model((self.dialog.flow, self.transaction)),
        }
    }
}

impl From<models::Dialog> for DirtyDialogWithTransaction {
    fn from(model: models::Dialog) -> DirtyDialogWithTransaction {
        let (flow, transaction): (DialogFlow, DirtyTransaction) = from_model(model.flow);
        DirtyDialogWithTransaction {
            dialog: DirtyDialog {
                computed_id: Some(model.computed_id),
                call_id: Some(model.call_id),
                from_tag: Some(model.from_tag),
                to_tag: Some(model.to_tag),
                flow: Some(flow),
                ..Default::default()
            },
            transaction,
        }
    }
}

fn into_model(tuple: (DialogFlow, Transaction)) -> models::DialogFlow {
    match tuple.0 {
        DialogFlow::Registration => models::DialogFlow::Registration(tuple.1.into()),
        DialogFlow::Invite => models::DialogFlow::Invite(tuple.1.into()),
        DialogFlow::Publish => models::DialogFlow::Publish(tuple.1.into()),
    }
}

fn from_model(model: models::DialogFlow) -> (DialogFlow, DirtyTransaction) {
    match model {
        models::DialogFlow::Registration(transaction) => {
            (DialogFlow::Registration, transaction.into())
        }
        models::DialogFlow::Invite(transaction) => (DialogFlow::Invite, transaction.into()),
        models::DialogFlow::Publish(transaction) => (DialogFlow::Publish, transaction.into()),
    }
}

fn computed_id_for(call_id: &str, from_tag: &str, to_tag: &Uuid) -> String {
    format!("{}-{}-{}", call_id, from_tag, to_tag)
}

fn flow_for_method(method: Method) -> Result<DialogFlow, String> {
    match method {
        Method::Register => Ok(DialogFlow::Registration),
        Method::Invite => Ok(DialogFlow::Invite),
        Method::Publish => Ok(DialogFlow::Publish),
        Method::Subscribe => Ok(DialogFlow::Publish),
        _ => Err("Unsupported method".into()),
    }
}

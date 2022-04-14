## Samples
### /assignments
#### Assignment attributes (returned by endpoints that return Assignments)
```
Assignment: {
   id: Uuid
   user_id: Uuid
   encoded_input: base64 encoded string
   encoded_output: base64 encoded string
   updated: timestamp
}
```
#### Create new Assignment
```
POST /assignments
workload: {
   id: Uuid
   user_id: Uuid
   encoded_input: base64 encoded string
   encoded_output: base64 encoded string
}
```
#### Get assignment by id
```
GET /assignment/{assignment_id}
```
####
### /submissions
#### Submission attributes (returned by endpoints that return Submissions)
```
Submission: {
   id: Uuid
   assignment_id: Uuid
   user_id: Uuid
   extension: String
   created: Timestamp
   update_count: int
}
```
#### Get all submission entries (debugging endpoint)
```
GET /submissions/
```
#### Get one Submission by user_id and assignment_id
```
GET /submissions?user_id={user_id}&assignment_id={assignment_id}
```
#### Get all submissions by assignment_id
```
GET /submissions?assignment_id={assignment_id}
```
#### Create new submission
```
POST /submissions
workload: {
   assignment_id: Uuid
   user_id: Uuid
   extension: String
   encoded_text: base64 encoded string
}
```
**BEWARE** this endpoint returns the file_id as well as all the fields of the submission!
Returns:
```
Submission: {
   id: Uuid
   assignment_id: Uuid
   user_id: Uuid
   extension: String
   created: Timestamp
   update_count: int
   file_id: Uuid
}
```
### /files
#### File attributes (returned by endpoints that return Files)
```
File: {
   id: Uuid
   submission_id: Uuid
   updated: Timestamp
   encoded_text: base64 encoded text
   scheduled: bool
   validated: bool
   encoded_output: base64 encoded text
}
```
#### Get file by uuid
```
GET /files/{file_id}
```
#### Get latest file by submission_id
```
GET /files?submission_id={submission_id}
```
#### Send output to repo service
```
PATCH /files/{file_id}
workload: {
   encoded_output: String (base64 encoded)
}
```
#### Trigger schedule of code
```
PATCH /files/{file_id}
workload: {
   id: Uuid
   scheduled: bool
}
```
## Frontend development setup
### Docker image pull
**Prequisite: installed docker, have gcloud logged in. [see here for login](https://cloud.google.com/container-registry/docs/advanced-authentication)**

0. set env variables with version that you want to use and some password:
   `REPO_VERSION=0.3.0 PG_PW=secret_pw`
1. run `gcloud auth configure-docker` to configure pull access for gcloud docker repo
2. pull desired version `docker pull gcr.io/autogradr-294411/repository-service:$REPO_VERSION`
3. create api.env with the following content:
```
DATABASE_URL=postgres://repository-service:$PG_PW@database:5432/repository-service
SCHEDULING_SUBMISSION_URL=scheduling-service/api/v0/submissions
 ```
4. create database.env with the following content:
```
POSTGRES_USER=repository-service
POSTGRES_PASSWORD=$PG_PW
POSTGRES_DB=repository-service
```
6. `echo REPO_VERSION=$REPO_VERSION >> compose.env`
5. run `docker-compose --env-file compose.env up -d`
6. run `docker-compose --env-file compose.env down` if you want to shut it down or if you have troubles
7. run `docker run -p 5432:5432 --env-file=database.env postgres` to run the database without the api

## Development setup
### Initialization

**rocket.rs** currently needs nightly rust, therefore you have to set rustc to nightly:

`rustup override set nightly`

These packages have to be installed to compile the app: `sudo apt install postgresql-13 libpq-dev libssl-dev`

To create the database, run `cargo install diesel_cli --no-default-features --features postgres` and then `diesel setup`.

A `.env` file with the DATABASE_URL variable in it has to be present for the setup on the dev machine.
```
DATABASE_URL=postgres://repository-service:{pg_pw}@127.0.0.1:5432/repository-service
ROCKET_ADDRESS=localhost
ROCKET_PORT=8888
SCHEDULING_SUBMISSION_URL=scheduling-service/api/v0/submissions
```
In production, this env variable will be set by the secrets manager.

### Database usage
link: [Kubernetes and Cloud SQL](https://cloud.google.com/sql/docs/postgres/connect-kubernetes-engine#proxy-with-service-account-key)

#### Setup
##### Create database and user

Create the user and the database for your service in the cloud sql admin page.

##### Add sidecar
First, you have to set your Pod or service to use the proxy as a sidecar. This can be achieved by
adding the container cloud-sql-proxy that is in `kubernetes/deployment.yml` to your own configuration.
The only thing that has to be changed there is perhaps the port, if you want it to run on a different port.
##### Provide key to sidecar
The sidecar needs the service account key to be able to connect to the database. For that, a kubectl secret is
added that can then be accessed by the container via a volume, that mounts the key.

The keyfile is currently set in github actions as a secret but can also be provided with a secret manager.

##### Use connection
The sidecar exposes the database on `127.0.0.1` and you can connect with the user created in cloud sql to it.
The port is declared in the sidecar container specification (see `kubernetes/deployment.yml`).

##### inject variables
The Environment variables used in the services can be injected via `./kustomize edit add secret [...]`
(see `.github/workflows/pipeline.yml` at the end).

Beware: `kustomize` has to be downloaded first!

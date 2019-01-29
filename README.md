# d2l-sync
Sync user accounts to d2l lms

A Rust based docker application to keep Desire 2 Learn LMS user information in sync with our backend data source. An event journal is monitored for user updates which trigger a synchronization process for the associated users.

## Environment variables:
* `D2L_APP_ID` and `D2L_APP_KEY` These are the application id and key used to sign requests. The application id and generated signature are included as query arguments in the request.
* `D2L_USR_ID` and `D2L_USR_KEY` These are the user id and key used to sign requests.
* `D2L_URI_BASE` This is the uri address to d2l for example `https://school_id.brightspace.com`.
* `D2L_JOURNAL_LIMIT` This limits the number of users retrieved with updated journal entries.
* `D2L_JOURNAL_ID_FILE` This is the location where the current journal id will be stored and loaded upon starup. If upon startup this file is not found the latest journal sequence number will be pulled from the journal and the process will start looking for updates from that point going forward.
* `D2L_SOURCE` This is a uri with information of a read only account to a backend database used as the source for syncing user information to d2l. An example would be `mysql://usr:pwd@host:port/db`.
* `D2L_QUERY_JOURNAL_MAX_ID` This is the query used to obtain the latest journal sequence number.
* `D2L_QUERY_JOURNAL` This is the query used to pull a list of distinct internal user id and associated journal sequence numbers up to `D2L_JOURNAL_LIMIT` of updated users starting at the current journal sequence number which is periodically saved within the `D2L_JOURNAL_ID_FILE`.
* `D2L_QUERY_USER` This is the query used to gather a user's information via their internal user id.

## Command-line options:
* `-i` | `--ids` This option is used to provide a comma delimited list of internal user id's that we may wish to sync to d2l.
* `-d` | `--data` This option is used to provide a json value filled with a users information you may wish to send to d2l. NOTE if you use this field you should also provide a role value for the -r option otherwise the default Student value will be used. An example value for this data option would `{"FirstName":"John","MiddleName":"","LastName":"Doe","UserName":"j_d1","OrgDefinedId":"X00000000","ExternalEmail":"jdoe@txstate.edu"}`
* `-r` | `--role` This is the role used when a value of the data option is used to create an account on d2l. The accepted values are Faculty, Staff, and Student. The default value is Student.

## NOTES:
A service account must be created via the D2L UI from which long lived application and user id/keys may be generated. The application and user keys are each are used to sign requests. The following is a command-line example which generates a non padded url safe base64 encoded SHA256 HMAC signature for a request:

```
KEY=<user or application key>
METHOD=GET
URL_PATH=/d2l/api/lp/1.20/users/214
EPOCH=$(date +%s)
echo -ne "$METHOD&URL_PATH&$EPOCH" |
  openssl sha256 -hmac $KEY -binary |
  openssl base64 -e |
  tr '+/' '-_' |
  tr -d '=\n'
```

# Secrets management

### The following environment variables needs to be set in order to install the secrets management using Google Secret Manager

```
export NAMESPACE=default
export CLUSTER_NAME=autogradr-294411-gke
export PROJECT_ID=autogradr-294411
export SECRETS_MANAGER_PROJECT_ID=autogradr-294411
export BUCKET_ID=autograder-secrets
export BUCKET_LOCATION=europe-west1
export SA_EMAIL=berglas-accessor@${PROJECT_ID}.iam.gserviceaccount.com
```

---

## Install secrets management to the cluster.
**This is a 1-time setup and does not need to be repeated unless we rebuild the cluster.**


Install [Berglas](https://github.com/GoogleCloudPlatform/berglas). There are binaries for Windows, Mac & Linux.

### Run the following commands:

Adapted from https://github.com/GoogleCloudPlatform/berglas/tree/main/examples/kubernetes

***Bootstrap the Berglas project (setup storage etc.)***
```
berglas bootstrap \
  --project $PROJECT_ID \
  --bucket $BUCKET_ID \
  --bucket-location $BUCKET_LOCATION
```

***Enable the cloud functions api***
```
  gcloud services enable --project ${PROJECT_ID} \
  cloudfunctions.googleapis.com
```

***Clone the berglas repo and go into the kubernetes folder***
 ```
 git clone git@github.com:GoogleCloudPlatform/berglas.git

 cd berglas/examples/kubernetes
```

***Deploy the mutating webhook as a cloud function***
```
  gcloud functions deploy berglas-secrets-webhook   --project ${PROJECT_ID}   --allow-unauthenticated   --runtime go113   --entry-point F   --trigger-http --region europe-west1
```

***Get the url of the cloud functiun endpoint***
```
  ENDPOINT=$(gcloud functions describe berglas-secrets-webhook --project ${PROJECT_ID} --region europe-west1 --format 'value(httpsTrigger.url)')
```
***Replace the placeholder URL with the $ENDPOINT url***
```
sed "s|REPLACE_WITH_YOUR_URL|$ENDPOINT|" deploy/webhook.yaml | kubectl apply -f -
```

***Verify that the cloud function endpoint has been deployed***
```
kubectl get mutatingwebhookconfiguration
```
Should return:
```
NAME
berglas-secrets-webhook
```

***Create a service account that has access to the secrets in Google Secret Manager***

```
  gcloud iam service-accounts create berglas-accessor \
  --project ${PROJECT_ID} \
  --display-name "Berglas secret accessor account"
```

```
  kubectl create serviceaccount "envserver"
```

```
  gcloud iam service-accounts add-iam-policy-binding \
  --project ${PROJECT_ID} \
  --role "roles/iam.workloadIdentityUser" \
  --member "serviceAccount:${PROJECT_ID}.svc.id.goog[default/envserver]" \
  berglas-accessor@${PROJECT_ID}.iam.gserviceaccount.com
```

```
  kubectl annotate serviceaccount "envserver" \
  iam.gke.io/gcp-service-account=berglas-accessor@${PROJECT_ID}.iam.gserviceaccount.com
```
That should complete the setup.

---
<br />
<br />

# Creating secrets

1. Create secret
`berglas create sm://${PROJECT_ID}/{name_of_secret} {secret_data}`
2. Let service account access secret
`berglas grant sm://${PROJECT_ID}/{name_of_secret} --member serviceAccount:$SA_EMAIL`

or 

1. Create secret from file
`berglas create sm://${PROJECT_ID}/{name_of_secret} @/path/to/file`
2. Let service account access secret
`berglas grant sm://${PROJECT_ID}/{name_of_secret} --member serviceAccount:$SA_EMAIL`

---
<br />
<br />

# Using secrets in kubernetes

***The secrets will be available as environment variables in the container and can thus be used from any programming language.***

For the secrets to be available, you need to add a few lines of configuration to your  `deployment.yaml` and Github Actions workflow files.

1. Add step to Github Actions file just before the `Deploy` step. *Replaces "PROJECT_ID" with actual Project ID*
```    - name: Add to project_id secrets management
      run: |-
        sed -i "s/sm:\/\/PROJECT_ID\//sm:\/\/${{ secrets.GKE_PROJECT }}\//g" deployment.yml
``` 
  See [example](https://github.com/Autogradr/mail-service/blob/20f680106c81e4c4961eb34f9b5bd9a6e7e9d34d/.github/workflows/mail-service.yml#L120)

2. Add the service account to the spec, just before the `containers:` line.
Add `serviceAccountName: envserver`. See [example](https://github.com/Autogradr/mail-service/blob/c5a35922d3f7d5c5d94dd8bb7b97844d50e0576a/deployment.yml#L20)

3. Add needed secrets as enviromnent variables to container:

For instance: 
  ```
          env:
            - name: MAILJET_API_KEY
              value: sm://PROJECT_ID/mail_service_api_key
            - name: MAILJET_API_SECRET
              value: sm://PROJECT_ID/mail_service_api_secret
  ``` 
  See [example](https://github.com/Autogradr/mail-service/blob/c5a35922d3f7d5c5d94dd8bb7b97844d50e0576a/deployment.yml#L25)


***Secrets are now available as environment variables for use in the code after deployment***

***Optional***
If you need any of the secrets available during the Github Actions steps, you can add them like this: [Secrets in Github Actions Step](https://github.com/Autogradr/mail-service/blob/20f680106c81e4c4961eb34f9b5bd9a6e7e9d34d/.github/workflows/mail-service.yml#L47)

# Ansible - Managing Kubernetes Cluster

## Install and configure GCloud

First, install the [Google Cloud CLI](https://cloud.google.com/sdk/docs/quickstarts) 
and initialize it.

```shell
$ gcloud init
```

Once you've initialized gcloud (signed in, selected project), add your account 
to the Application Default Credentials (ADC). This will allow Terraform to access
these credentials to provision resources on GCloud.

```shell
$ gcloud auth login
```

## Setup key

This is a key that gives you access to a service account in the GCP cluster. 
The key file is available by requesting it from Morten, Petur or Donat. We should figure out a common place to store this.

```shell
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/credentials.json"
```

## Initialize Ansible

### Install Ansible
Make sure you have `python` and `pip` installed.

First, install [Ansible](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html#installing-ansible-on-ubuntu).


## Run Ansible

To configure kubetcl, by running the following command. 

```shell
$ gcloud container clusters get-credentials autogradr-294411-gke --region europe-west1
```

It generates kubeconfig certificates at location `HOME/.kube/config`

This certificate is then used to spawn up Ansible against the cluster

Run the playbook from the ansible folder

```shell
$ ansible-playbook playbook.main.yml -vvv
```

## Open OpenFaas client

Get the external IP of Open Faas Gateway

```shell
kubectl get -n openfaas svc/gateway-external
```

Copy the IP in browser with PORT 8080

Next, you're promptet with username and password
Username is `Admin`

To get the OpenFaas password
```shell
kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo
```
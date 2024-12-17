# Infrastructure

# Ansible

## hosts.yaml

The inventory file for the ansible playbooks. This contains variable for each server.

## playbook.yaml

The main playbook that configures the servers

```
ansible-playbook -i hosts.yaml playbook.yaml
```

After running this you will see an `output/k3s.yaml` file which is the kubeconfig for the cluster.

You need to have the following environment variables set:

```
K3S_JOIN_TOKEN (always the same)
TAILSCALE_TOKEN
```

## uninstall.yaml

To remove all the services and configurations from the servers.

WARNING: THIS WILL DELETE EVERYTHING.

```
ansible-playbook -i hosts.yaml uninstall.yaml
```

## ssh-fingerprints.sh

This script regenerates the SSH fingerprints for the servers.

```
./ssh-fingerprints.sh
```

# Kubernetes

We have a bunch of helm charts in the `cluster/helm` directory which are used to deploy the services to the kubernetes cluster.

We also have manual scripts in the `cluster/kubectl` directory which are used to manage the kubernetes cluster.

# Tailscale

Each server in the cluster uses tailscale and we use tailscale to connect to the servers. For k8s we have a loadbalancer server in `hcloud` that is accessible from the `k3s-control-plane-loadbalancer` hostname in the tailscale network.

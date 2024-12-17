# Infrastructure

We use a kubernetes cluster (k3s) to orchestrate the services.

We use ansible to manage and configure these servers remotely.

Look at the [README.md](../infrastructure/README.md) for more information.

## Hardware

Each server has its own hardware configured;

The database nodes are AX162-R with 2 additional 960GB NVME drives.

The compute nodes are AX102.

The CDN nodes are EX44.

The loadbalancer nodes are EX44.

The event api nodes are EX44 with 128GB RAM.

The monitoring nodes are EX44

The control plane nodes are EX44.

Each server has a 10Gbit/s internal network connection to a 48 port 10Gbit Switch

The switch can be accessed from `192.168.0.254/24` using the password provided by the Hetzner team. Although you basically never need to access this.

Each server has a 1Gbit/s external network connection to the internet. 

### Subnets

- `10.0.0.0/16` - Node IPv4 CIDR
- `fc00:cafe:0000::/48` - Node IPv6 CIDR
- `10.42.0.0/16` - Cluster IPv4 CIDR (used for kubernetes pods)
- `2001:cafe:42::/56` - Cluster IPv6 CIDR (used for kubernetes pods)
- `10.43.0.0/16` - Service IPv4 CIDR (used for kubernetes services)
- `2001:cafe:43::/112` - Service IPv6 CIDR (used for kubernetes services)

## Kubernetes

We use a kubernetes cluster (k3s) to manage the servers.

Our cluster consists of the following node pools:

- `control_plane` - Control plane nodes (3 nodes)
- `database` - Database nodes (3 nodes)
- `compute` - Compute nodes (3 nodes)
- `cdn` - CDN nodes (10 nodes)
- `loadbalancer` - Loadbalancer nodes (3 nodes)
- `event_api` - Event API nodes (6 nodes)
- `monitoring` - Monitoring nodes (3 nodes)

### Control Plane

The control plane nodes are the nodes that run the control plane components.

### Database

The database nodes are the nodes that run the database components.

Such as redis, mongodb, clickhouse, typesense, etc.


### Compute

General compute nodes used to run the API, Website, etc.

### CDN

CDN nodes only run the `cdn` service.

### Loadbalancer

Loadbalancer nodes only run `cloudflared`.

### Event API

Event API nodes only run `event-api` service.

### Monitoring

Monitoring nodes run all the monitoring services such as `victoria-metrics`, `tempo`, `grafana`, etc.

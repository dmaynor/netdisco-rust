-- Netdisco-Rust initial schema
-- Based on the original Perl Netdisco schema (version 96)

CREATE TABLE IF NOT EXISTS device (
    ip           inet PRIMARY KEY,
    creation     TIMESTAMP DEFAULT LOCALTIMESTAMP,
    dns          text,
    description  text,
    uptime       bigint,
    contact      text,
    name         text,
    location     text,
    layers       varchar(8),
    ports        integer,
    mac          macaddr,
    serial       text,
    model        text,
    ps1_type     text,
    ps2_type     text,
    ps1_status   text,
    ps2_status   text,
    fan          text,
    slots        integer,
    vendor       text,
    os           text,
    os_ver       text,
    log          text,
    snmp_ver     integer,
    snmp_comm    text,
    snmp_class   text,
    vtp_domain   text,
    last_discover TIMESTAMP,
    last_macsuck  TIMESTAMP,
    last_arpnip   TIMESTAMP,
    pae_is_enabled boolean,
    custom_fields jsonb NOT NULL DEFAULT '{}',
    tags         text[] NOT NULL DEFAULT '{}'::text[]
);

CREATE INDEX IF NOT EXISTS idx_device_dns ON device(dns);
CREATE INDEX IF NOT EXISTS idx_device_layers ON device(layers);
CREATE INDEX IF NOT EXISTS idx_device_vendor ON device(vendor);
CREATE INDEX IF NOT EXISTS idx_device_model ON device(model);

CREATE TABLE IF NOT EXISTS device_ip (
    ip          inet,
    alias       inet,
    subnet      cidr,
    port        text,
    dns         text,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    PRIMARY KEY(ip, alias)
);

CREATE INDEX IF NOT EXISTS idx_device_ip_ip ON device_ip(ip);
CREATE INDEX IF NOT EXISTS idx_device_ip_alias ON device_ip(alias);

CREATE TABLE IF NOT EXISTS device_module (
    ip            inet NOT NULL,
    index         integer,
    description   text,
    type          text,
    parent        integer,
    name          text,
    class         text,
    pos           integer,
    hw_ver        text,
    fw_ver        text,
    sw_ver        text,
    serial        text,
    model         text,
    fru           boolean,
    creation      TIMESTAMP DEFAULT LOCALTIMESTAMP,
    last_discover TIMESTAMP,
    PRIMARY KEY(ip, index)
);

CREATE TABLE IF NOT EXISTS device_port (
    ip           inet,
    port         text,
    creation     TIMESTAMP DEFAULT LOCALTIMESTAMP,
    descr        text,
    up           text,
    up_admin     text,
    type         text,
    duplex       text,
    duplex_admin text,
    speed        text,
    name         text,
    mac          macaddr,
    mtu          integer,
    stp          text,
    remote_ip    inet,
    remote_port  text,
    remote_type  text,
    remote_id    text,
    vlan         text,
    pvid         integer,
    lastchange   bigint,
    ifindex      integer,
    is_uplink    boolean,
    speed_admin  text,
    is_master    boolean,
    slave_of     text,
    custom_fields jsonb NOT NULL DEFAULT '{}',
    tags         text[] NOT NULL DEFAULT '{}'::text[],
    PRIMARY KEY(port, ip)
);

CREATE INDEX IF NOT EXISTS idx_device_port_ip ON device_port(ip);
CREATE INDEX IF NOT EXISTS idx_device_port_remote_ip ON device_port(remote_ip);
CREATE INDEX IF NOT EXISTS idx_device_port_mac ON device_port(mac);

CREATE TABLE IF NOT EXISTS device_port_log (
    id          serial,
    ip          inet,
    port        text,
    reason      text,
    log         text,
    username    text,
    userip      inet,
    action      text,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_device_port_log_1 ON device_port_log(ip, port);

CREATE TABLE IF NOT EXISTS device_port_power (
    ip          inet,
    port        text,
    module      integer,
    admin       text,
    status      text,
    class       text,
    power       integer,
    PRIMARY KEY(port, ip)
);

CREATE TABLE IF NOT EXISTS device_port_ssid (
    ip          inet,
    port        text,
    ssid        text,
    broadcast   boolean,
    bssid       macaddr
);

CREATE INDEX IF NOT EXISTS idx_device_port_ssid_ip_port ON device_port_ssid(ip, port);

CREATE TABLE IF NOT EXISTS device_port_vlan (
    ip          inet,
    port        text,
    vlan        integer,
    native      boolean NOT NULL DEFAULT false,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    last_discover TIMESTAMP DEFAULT LOCALTIMESTAMP,
    vlantype    text,
    PRIMARY KEY(ip, port, vlan)
);

CREATE TABLE IF NOT EXISTS device_port_wireless (
    ip          inet,
    port        text,
    channel     integer,
    power       integer
);

CREATE INDEX IF NOT EXISTS idx_device_port_wireless_ip_port ON device_port_wireless(ip, port);

CREATE TABLE IF NOT EXISTS device_power (
    ip          inet,
    module      integer,
    power       integer,
    status      text,
    PRIMARY KEY(ip, module)
);

CREATE TABLE IF NOT EXISTS device_vlan (
    ip          inet,
    vlan        integer,
    description text,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    last_discover TIMESTAMP DEFAULT LOCALTIMESTAMP,
    PRIMARY KEY(ip, vlan)
);

CREATE TABLE IF NOT EXISTS device_skip (
    backend     text,
    device      inet NOT NULL,
    actionset   text[],
    deferrals   integer,
    last_defer  TIMESTAMP
);

CREATE TABLE IF NOT EXISTS admin (
    job         serial,
    entered     TIMESTAMP DEFAULT LOCALTIMESTAMP,
    started     TIMESTAMP,
    finished    TIMESTAMP,
    device      inet,
    port        text,
    action      text,
    subaction   text,
    status      text,
    username    text,
    userip      inet,
    log         text,
    debug       boolean
);

CREATE INDEX IF NOT EXISTS idx_admin_entered ON admin(entered);
CREATE INDEX IF NOT EXISTS idx_admin_status ON admin(status);
CREATE INDEX IF NOT EXISTS idx_admin_action ON admin(action);

CREATE TABLE IF NOT EXISTS node (
    mac         macaddr,
    switch      inet,
    port        text,
    vlan        text DEFAULT '0',
    active      boolean,
    oui         varchar(8),
    time_first  TIMESTAMP DEFAULT LOCALTIMESTAMP,
    time_recent TIMESTAMP DEFAULT LOCALTIMESTAMP,
    time_last   TIMESTAMP DEFAULT LOCALTIMESTAMP,
    PRIMARY KEY(mac, switch, port, vlan)
);

CREATE INDEX IF NOT EXISTS idx_node_switch_port ON node(switch, port);
CREATE INDEX IF NOT EXISTS idx_node_mac ON node(mac);
CREATE INDEX IF NOT EXISTS idx_node_mac_active ON node(mac, active);

CREATE TABLE IF NOT EXISTS node_ip (
    mac         macaddr,
    ip          inet,
    active      boolean,
    dns         text,
    time_first  TIMESTAMP DEFAULT LOCALTIMESTAMP,
    time_last   TIMESTAMP DEFAULT LOCALTIMESTAMP,
    PRIMARY KEY(mac, ip)
);

CREATE INDEX IF NOT EXISTS idx_node_ip_ip ON node_ip(ip);
CREATE INDEX IF NOT EXISTS idx_node_ip_mac ON node_ip(mac);

CREATE TABLE IF NOT EXISTS node_monitor (
    mac         macaddr PRIMARY KEY,
    active      boolean,
    why         text,
    cc          text,
    date        TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE TABLE IF NOT EXISTS node_nbt (
    mac         macaddr PRIMARY KEY,
    ip          inet,
    nbname      text,
    domain      text,
    server      boolean,
    nbuser      text,
    active      boolean,
    time_first  TIMESTAMP DEFAULT LOCALTIMESTAMP,
    time_last   TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE TABLE IF NOT EXISTS node_wireless (
    mac         macaddr,
    ssid        text DEFAULT '',
    uptime      integer,
    maxrate     integer,
    txrate      integer,
    sigstrength integer,
    sigqual     integer,
    rxpkt       integer,
    txpkt       integer,
    rxbyte      bigint,
    txbyte      bigint,
    time_last   TIMESTAMP DEFAULT LOCALTIMESTAMP,
    PRIMARY KEY(mac, ssid)
);

CREATE TABLE IF NOT EXISTS log (
    id          serial,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    class       text,
    entry       text,
    logfile     text
);

CREATE TABLE IF NOT EXISTS oui (
    oui         varchar(8) PRIMARY KEY,
    company     text
);

CREATE TABLE IF NOT EXISTS process (
    controller  integer NOT NULL,
    device      inet NOT NULL,
    action      text NOT NULL,
    status      text,
    count       integer,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    id          char(32) NOT NULL PRIMARY KEY,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    a_session   text
);

CREATE TABLE IF NOT EXISTS subnets (
    net         cidr NOT NULL PRIMARY KEY,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    last_discover TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE TABLE IF NOT EXISTS topology (
    dev1        inet NOT NULL,
    port1       text NOT NULL,
    dev2        inet NOT NULL,
    port2       text NOT NULL
);

CREATE TABLE IF NOT EXISTS user_log (
    entry       serial,
    username    varchar(50),
    userip      inet,
    event       text,
    details     text,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
    username    varchar(50) PRIMARY KEY,
    password    text,
    creation    TIMESTAMP DEFAULT LOCALTIMESTAMP,
    last_on     TIMESTAMP,
    port_control boolean DEFAULT false,
    ldap        boolean DEFAULT false,
    admin       boolean DEFAULT false,
    fullname    text,
    note        text
);

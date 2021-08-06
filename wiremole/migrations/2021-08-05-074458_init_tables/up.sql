CREATE TABLE `interfaces` (
    `id` INT AUTO_INCREMENT PRIMARY KEY,
    `devname` VARCHAR(15) NOT NULL UNIQUE KEY,
    -- one-to-many interface_ips
    `mtu` INT UNSIGNED,
    `privkey` BINARY(32),
    `fwmark` INT UNSIGNED NOT NULL DEFAULT '0',
    `listen_port` SMALLINT UNSIGNED NOT NULL DEFAULT '0'
    -- one-to-many peers
);

CREATE TABLE `interface_ips` (
    `id` INT AUTO_INCREMENT PRIMARY KEY,
    `interface_id` INT,
    `ipaddress` VARBINARY(16) NOT NULL,
    `mask` TINYINT UNSIGNED NOT NULL,
    FOREIGN KEY(`interface_id`) REFERENCES `interfaces`(`id`) ON DELETE CASCADE
);

CREATE TABLE `peers` (
    `id` INT AUTO_INCREMENT PRIMARY KEY,
    `interface_id` INT,
    `pubkey` BINARY(32) NOT NULL,
    `preshared_key` BINARY(32),
    -- endpoint
    `endpoint_ip` VARBINARY(16),
    `endpoint_port` SMALLINT UNSIGNED,
    `endpoint_flowinfo` INT UNSIGNED,
    -- end endpoint
    `persistent_keepalive` SMALLINT UNSIGNED,
    -- one-to-many allowed_ips
    CONSTRAINT `uc_peer` UNIQUE KEY (`interface_id`, `pubkey`),
    FOREIGN KEY(`interface_id`) REFERENCES `interfaces`(`id`) ON DELETE CASCADE
);

CREATE TABLE `allowed_ips` (
    `id` INT AUTO_INCREMENT PRIMARY KEY,
    `peer_id` INT,
    `ipaddress` VARBINARY(16) NOT NULL,
    `mask` TINYINT UNSIGNED NOT NULL,
    FOREIGN KEY(`peer_id`) REFERENCES `peers`(`id`) ON DELETE CASCADE
);

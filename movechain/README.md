# SupplyChain Module for IOTA Blockchain

The `SupplyChain` module is part of the `MoveChain` project, designed to manage ownership of a product in the supply chain processes on the IOTA blockchain utilizing the Move programming language. This module offers functionalities for transaction handling, state management, and environmental data tracking to enhance transparency and efficiency in supply chains. 

This tutorial provides step-by-step instructions to install the `SupplyChain` module for the IOTA blockchain. These instructions are intended for execution in a typical blockchain project environment.

## Features

- **Product Lifecycle Management**: The system will monitor the entire lifecycle of a product from production to distribution.
- **State Management**: Manages the transition of products from being owned to being shared state.
- **Event Logging**: Records the state variation and other important events to ensure traceability and auditability.
- **Environmental Data Tracking**: Records and updates environmental conditions associated with products.
- **Role-Based Access Control**: Ensures that operations are only performed by authorized participants.

## Prerequisites

Before starting, ensure that you have the IOTA blockchain environment set up and that you have Git installed to manage the versions of your project files.

## Installation

To integrate the `SupplyChain` module into your existing IOTA blockchain project:

### 1. Clone the Repository

Ensure you have Git installed on your system. You can verify the presence of Git with the following command:

```bash
git --version
```

If Git is not installed, you can install it with the following command:

```bash
sudo apt-get install git
```

### 2. Clone the MoveChain Repository
Visit the GitHub page of the MoveChain project and copy the cloning link. Here is an example of the command to clone the repository:

```bash
git clone https://github.com/SimoGiuffrida/MoveChain.git
cd MoveChain
```

### 3. Include the SupplyChain Module
Navigate to the directory where you want to include the SupplyChain module, and ensure it is placed correctly within your project structure. You might need to adjust paths depending on your specific environment setup.

## Usage
After installation, you can begin to use the SupplyChain module within your IOTA blockchain projects. Below are some examples of how to utilize the module to create and manage products in the supply chain.

### Creating a Product
To create a new product within the supply chain, use the following function:
```bash
<PackageID>::SupplyChain::create_product <min_sensor_data> <max_sensor_data>
```

### Assigning a Distributor
To assign a distributor to a product, use the following function:
```bash
SupplyChain::assign_distributor  <Product_ID> <distributor_address>
```

### Confirming Product Delivery
To confirm the delivery of a product and transfer ownership to the buyer, the buyer will have to use the following function:
```bash
SupplyChain::confirm_delivery <Product_ID>
```

## Contributing
Contributions to the MoveChain project are welcome. Please adhere to the project's coding standards and include appropriate tests for all new code.


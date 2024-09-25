import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const WIOTAModule = buildModule("WIOTAModule", (m) => {

  const wiota = m.contract("WIOTA", []);

  return { wiota };
});

export default WIOTAModule;

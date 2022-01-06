import React, { createContext, useState } from "react";

export const ConfirmContext = createContext();

const ConfirmContextProvider = ({ children }) => {
  const [confirm, setConfirm] = useState({
    prompt: "",
    isOpen: false,
    proceed: null,
    cancel: null
  });

  return (
    <ConfirmContext.Provider value={[confirm, setConfirm]}>
      {children}
    </ConfirmContext.Provider>
  );
};

export default ConfirmContextProvider;
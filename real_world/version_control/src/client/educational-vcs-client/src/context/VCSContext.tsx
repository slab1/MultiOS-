import React, { createContext, useContext } from 'react';

const VCSContext = createContext();

export const useVCS = () => {
  const context = useContext(VCSContext);
  if (!context) {
    throw new Error('useVCS must be used within a VCSProvider');
  }
  return context;
};

export const VCSProvider = ({ children, value }) => {
  return (
    <VCSContext.Provider value={value}>
      {children}
    </VCSContext.Provider>
  );
};

import type {
    ReactNode,
  } from "react";
  
  import {
    Sidebar,
  } from "./Sidebar";
  
  import type {
    AppPage,
  } from "./Sidebar";
  
  type AppShellProps = {
    children:
      ReactNode;
  
    activePage:
      AppPage;
  
    onNavigate:
      (
        page: AppPage,
      ) => void;
  
    hasDataset:
      boolean;
  };
  
  export function AppShell(
    {
      children,
      activePage,
      onNavigate,
      hasDataset,
    }: AppShellProps,
  ) {
    return (
      <div className="app-shell">
        <Sidebar
          activePage={
            activePage
          }
          onNavigate={
            onNavigate
          }
          hasDataset={
            hasDataset
          }
        />
  
        <main className="app-content">
          {children}
        </main>
      </div>
    );
  }
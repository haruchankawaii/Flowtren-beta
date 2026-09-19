import {
  useState,
} from "react";

import {
  AppShell,
} from "../components/layout/AppShell";

import type {
  AppPage,
} from "../components/layout/Sidebar";

import {
  DatasetProvider,
  useDataset,
} from "../features/dataset/context";

import {
  CleanPage,
} from "../pages/CleanPage";

import {
  ExplorePage,
} from "../pages/ExplorePage";

import {
  ExportPage,
} from "../pages/ExportPage";

import {
  HomePage,
} from "../pages/HomePage";

import {
  InsightsPage,
} from "../pages/InsightsPage";

function Application() {
  const [
    activePage,
    setActivePage,
  ] =
    useState<AppPage>(
      "home",
    );

  const [
    visitedPages,
    setVisitedPages,
  ] =
    useState<
      Set<AppPage>
    >(
      () =>
        new Set<AppPage>([
          "home",
        ]),
    );

  const {
    dataset,
  } =
    useDataset();

  function handleNavigate(
    page:
      AppPage,
  ) {
    setVisitedPages(
      (current) => {
        if (
          current.has(
            page,
          )
        ) {
          return current;
        }

        const next =
          new Set(
            current,
          );

        next.add(
          page,
        );

        return next;
      },
    );

    setActivePage(
      page,
    );
  }

  return (
    <AppShell
      activePage={
        activePage
      }
      onNavigate={
        handleNavigate
      }
      hasDataset={
        dataset !== null
      }
    >
      <PageContainer
        page="home"
        activePage={
          activePage
        }
        visitedPages={
          visitedPages
        }
      >
        <HomePage />
      </PageContainer>

      <PageContainer
        page="clean"
        activePage={
          activePage
        }
        visitedPages={
          visitedPages
        }
      >
        <CleanPage />
      </PageContainer>

      <PageContainer
        page="insights"
        activePage={
          activePage
        }
        visitedPages={
          visitedPages
        }
      >
        <InsightsPage />
      </PageContainer>

      <PageContainer
        page="explore"
        activePage={
          activePage
        }
        visitedPages={
          visitedPages
        }
      >
        <ExplorePage />
      </PageContainer>

      <PageContainer
        page="export"
        activePage={
          activePage
        }
        visitedPages={
          visitedPages
        }
      >
        <ExportPage />
      </PageContainer>
    </AppShell>
  );
}

type PageContainerProps = {
  page:
    AppPage;

  activePage:
    AppPage;

  visitedPages:
    Set<AppPage>;

  children:
    React.ReactNode;
};

function PageContainer(
  {
    page,
    activePage,
    visitedPages,
    children,
  }: PageContainerProps,
) {
  if (
    !visitedPages.has(
      page,
    )
  ) {
    return null;
  }

  return (
    <div
      hidden={
        activePage
        !== page
      }
    >
      {children}
    </div>
  );
}

export default function App() {
  return (
    <DatasetProvider>
      <Application />
    </DatasetProvider>
  );
}
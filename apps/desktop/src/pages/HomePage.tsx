import {
  useDataset,
} from "../features/dataset/context";

export function HomePage() {
  const {
    dataset,
    openingDataset,
    loading,
    analysisStatus,
    error,
    analysisError,
    openSpreadsheet,
  } =
    useDataset();

  if (
    !dataset
    && !openingDataset
  ) {
    return (
      <section className="home-page">
        <div className="hero">
          <p className="eyebrow">
            Private spreadsheet
            analysis
          </p>

          <h1>
            Your spreadsheet,
            finally easy to
            understand.
          </h1>

          <p className="hero-description">
            Open a CSV or Excel
            file. Flowtren analyzes
            everything locally on
            your computer.
          </p>

          <button
            type="button"
            className="primary-button"
            disabled={
              loading
            }
            onClick={
              () => {
                void openSpreadsheet();
              }
            }
          >
            {loading
              ? "Opening..."
              : "Choose spreadsheet"}
          </button>

          <p className="privacy-note">
            Your spreadsheet never
            leaves this device.
          </p>

          {error && (
            <div className="error-card">
              {error}
            </div>
          )}
        </div>
      </section>
    );
  }

  /*
   * open_dataset sudah selesai,
   * tapi profile / quality / cleaning
   * masih berjalan.
   */
  if (
    !dataset
    && openingDataset
  ) {
    return (
      <section className="home-page">
        <header className="page-header">
          <div>
            <p className="eyebrow">
              Dataset
            </p>

            <h1>
              {
                openingDataset
                  .fileName
              }
            </h1>

            <p className="dataset-path">
              {
                openingDataset
                  .path
              }
            </p>
          </div>

          <button
            type="button"
            className="secondary-button"
            disabled={
              loading
              || analysisStatus
              === "analyzing"
            }
            onClick={
              () => {
                void openSpreadsheet();
              }
            }
          >
            Open another file
          </button>
        </header>

        <div className="summary-grid">
          <article className="summary-card">
            <span className="summary-label">
              Rows
            </span>

            <strong>
              {openingDataset
                .rowCount
                .toLocaleString()}
            </strong>
          </article>

          <article className="summary-card">
            <span className="summary-label">
              Columns
            </span>

            <strong>
              {openingDataset
                .columnCount
                .toLocaleString()}
            </strong>
          </article>

          <article className="summary-card">
            <span className="summary-label">
              Quality
            </span>

            <strong>
              —
            </strong>
          </article>

          <article className="summary-card">
            <span className="summary-label">
              Cleaning suggestions
            </span>

            <strong>
              —
            </strong>
          </article>
        </div>

        {analysisStatus
          === "analyzing" && (
          <div className="content-card">
            <div className="card-heading">
              <div>
                <p className="eyebrow">
                  Analysis
                </p>

                <h2>
                  Analyzing dataset...
                </h2>
              </div>
            </div>

            <p className="page-description">
              Flowtren is profiling
              columns, checking data
              quality, and looking for
              cleaning suggestions
              locally on your computer.
            </p>
          </div>
        )}

        {analysisStatus
          === "error" && (
          <div className="error-card">
            {analysisError
              ?? "Dataset analysis failed."}
          </div>
        )}

        {error && (
          <div className="error-card">
            {error}
          </div>
        )}
      </section>
    );
  }

  /*
   * TypeScript belum otomatis tahu
   * dataset pasti non-null setelah dua
   * branch di atas, jadi guard eksplisit.
   */
  if (!dataset) {
    return null;
  }

  return (
    <section className="home-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">
            Dataset
          </p>

          <h1>
            {
              dataset.fileName
            }
          </h1>

          <p className="dataset-path">
            {
              dataset.path
            }
          </p>
        </div>

        <button
          type="button"
          className="secondary-button"
          disabled={
            loading
            || analysisStatus
            === "analyzing"
          }
          onClick={
            () => {
              void openSpreadsheet();
            }
          }
        >
          {loading
            ? "Opening..."
            : "Open another file"}
        </button>
      </header>

      <div className="summary-grid">
        <article className="summary-card">
          <span className="summary-label">
            Rows
          </span>

          <strong>
            {dataset
              .profile
              .rowCount
              .toLocaleString()}
          </strong>
        </article>

        <article className="summary-card">
          <span className="summary-label">
            Columns
          </span>

          <strong>
            {dataset
              .profile
              .columnCount
              .toLocaleString()}
          </strong>
        </article>

        <article className="summary-card">
          <span className="summary-label">
            Quality
          </span>

          <strong>
            {dataset
              .quality
              .score
              .toFixed(0)}
            /100
          </strong>
        </article>

        <article className="summary-card">
          <span className="summary-label">
            Cleaning suggestions
          </span>

          <strong>
            {
              dataset
                .cleaningSuggestions
                .length
            }
          </strong>
        </article>
      </div>

      <div className="content-card">
        <div className="card-heading">
          <div>
            <p className="eyebrow">
              Columns
            </p>

            <h2>
              Dataset overview
            </h2>
          </div>
        </div>

        <div className="table-wrapper">
          <table>
            <thead>
              <tr>
                <th>
                  Column
                </th>

                <th>
                  Type
                </th>

                <th>
                  Semantic type
                </th>

                <th>
                  Missing
                </th>

                <th>
                  Unique
                </th>
              </tr>
            </thead>

            <tbody>
              {dataset
                .profile
                .columns
                .map(
                  (
                    column,
                  ) => (
                    <tr
                      key={
                        column.name
                      }
                    >
                      <td>
                        {
                          column.name
                        }
                      </td>

                      <td>
                        {
                          column.dtype
                        }
                      </td>

                      <td>
                        {
                          column
                            .semanticType
                        }
                      </td>

                      <td>
                        {
                          column
                            .nullCount
                        }
                      </td>

                      <td>
                        {
                          column
                            .uniqueCount
                        }
                      </td>
                    </tr>
                  ),
                )}
            </tbody>
          </table>
        </div>
      </div>

      {analysisStatus
        === "analyzing" && (
        <div className="content-card">
          <p>
            Refreshing dataset
            analysis...
          </p>
        </div>
      )}

      {analysisError && (
        <div className="error-card">
          {analysisError}
        </div>
      )}

      {error && (
        <div className="error-card">
          {error}
        </div>
      )}
    </section>
  );
}
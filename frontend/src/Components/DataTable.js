import React from "react";
import PropTypes from "prop-types";
import DataTable, { createTheme } from "react-data-table-component";
import themes from "../App/theme";
import { ThemeContext } from "styled-components";

const Table = (props) => {
  const themeContext = React.useContext(ThemeContext);

  // Create a table theme for each waitlist theme
  Object.entries(themes).forEach((entry) => {
    const [key, value] = entry;
    createTheme(
      `TDF-${key}`,
      {
        text: {
          primary: value.colors.primary,
          secondary: value.colors.secondary,
        },
        background: {
          default: value.colors.background,
        },
      },
      value.base ?? "dark"
    );
  });

  return <DataTable {...props} theme={`TDF-${themeContext.name}`} />;
};

Table.propTypes = {
  columns: PropTypes.arrayOf(PropTypes.object).isRequired,
  customStyles: PropTypes.object,
  data: PropTypes.array,
  pagination: PropTypes.bool,
  paginationPerPage: PropTypes.number,
  paginationRowsPerPageOptions: PropTypes.arrayOf(PropTypes.number),
  persistTableHead: PropTypes.bool,
};

Table.defaultProps = {
  customStyles: {
    head: {
      style: {
        fontSize: "unset",
      },
    },
    subHeader: {
      style: {
        paddingLeft: "12px",
      },
    },
    rows: {
      style: {
        fontSize: "15px",
      },
    },
  },
  pagination: true,
  paginationPerPage: 50,
  paginationRowsPerPageOptions: [10, 25, 50, 75, 100],
  persistTableHead: true,
};

export default Table;

const SortAlphabetical = (a, b) => a.localeCompare(b, { sensitivity: "base" });

const MaxDate = new Date(8640000000000000);
const SortDate = (a, b) => {
  if (!a) a = MaxDate;
  if (!b) b = MaxDate;
  if (!(a instanceof Date)) a = new Date(a);
  if (!(b instanceof Date)) b = new Date(b);
  return a.getTime() - b.getTime();
};

const order = ["Character", "Corporation", "Alliance"];
const SortByEntityCategory = (a, b) => {
  if (order.indexOf(a.category) > order.indexOf(b.category)) {
    return 1;
  } else if (order.indexOf(a.category) < order.indexOf(b.category)) {
    return -1;
  }
  return 0;
};

export { SortAlphabetical, SortByEntityCategory, SortDate };

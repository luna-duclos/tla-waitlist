import styled from "styled-components";

export const Table = styled.table`
  table-layout: fixed;
  ${(props) => props.fullWidth && "width: 100%;"}
  ${(props) =>
    props.edgeLine &&
    `
    border-top: solid 1px ${props.theme.colors.accent2};
    border-bottom: solid 1px ${props.theme.colors.accent2};
  `}
  @media (max-width: 480px) {
    width: 100%;
    font-size: 3.4vw;
  }
`;

export const TableHead = styled.thead`
  border-bottom: solid 2px ${(props) => props.theme.colors.accent2};
`;

export const TableBody = styled.tbody``;

export const Row = styled.tr`
  background-color: ${(props) => props.theme.colors.background};
  ${(props) =>
    props.background &&
    `
    background-color: ${props.theme.colors.accent1};
    box-shadow: 2px 2px 5px ${props.theme.colors.shadow};
    
  `}

  &:last-child {
    border-bottom: none;
  }
  ${(props) =>
    !props.noAlternating &&
    `
    border-bottom: solid 1px ${props.theme.colors.accent2};
    &:nth-child(odd) {
      background-color: ${props.theme.colors.accent1};
    }
	
  `}
`;

export const CellHead = styled.th`
  text-align: left;
  padding: 0.5em;
  font-weight: 600;
`;

export const SmallCellHead = styled.th`
  text-align: left;
  padding: 0.5em;
  font-weight: 600;
  width: 75px;
  @media (max-width: 480px) {
    width: 60px;
  }
`;

export const Cell = styled.td`
  padding: 0.5em;
  color: ${(props) => props.theme.colors.text};
  a {
    color: ${(props) => props.theme.colors.text};
  }
  @media (max-width: 480px) {
    padding: 0.3em;
  }
`;

export const CellTight = styled.td`
  color: ${(props) => props.theme.colors.text};
  a {
    color: ${(props) => props.theme.colors.text};
  }
  @media (max-width: 480px) {
    padding: 0.3em;
  }
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-right: 0.3em;
`;

export const CellWithLine = styled.td`
  position: relative;
  text-align: end;
  padding: 0 0.5em;
  &::before {
    content: "";
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 2px;
    background-color: ${(props) => props.theme.colors.accent2};
  }

  ${(props) =>
    props.warn &&
    `
		  &::after {
			content: "";
			position: absolute;
			left: 40%;
			top: 0;
			bottom: 0;
			width: 10%;
			background: ${props.warn};
		  }
		`}
`;

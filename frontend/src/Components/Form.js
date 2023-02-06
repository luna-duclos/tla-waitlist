import styled, { css } from "styled-components";
import { NavLink } from "react-router-dom";

const inputStyle = css`
  position: relative;
  padding: 0 1em;
  font-size: 1em;
  border: solid ${(props) => (props.variant ? "0px" : "1px")};
  border-color: ${(props) => props.theme.colors.accent2};
  border-radius: 20px;
  background-color: ${(props) => props.theme.colors[props.variant || "input"].color};
  color: ${(props) => props.theme.colors[props.variant || "input"].text} !important;
  display: inline-block;
  font: inherit;
  &.active {
    border-color: ${(props) => props.theme.colors.accent2};
  }
  &:hover:not(:disabled):not(.static) {
    color: ${(props) => props.theme.colors[props.variant || "input"].text};
    border-color: ${(props) => props.theme.colors.accent3};
    background-color: ${(props) => props.theme.colors[props.variant || "input"].accent};
    &.active {
      border-color: ${(props) => props.theme.colors.active};
    }
  }
  &:focus:not(:disabled):not(.static) {
    outline: none;
  }
  &:disabled {
    cursor: not-allowed;
  }
  &.static {
    cursor: default;
  }
  &:disabled,
  &.static {
    color: ${(props) => props.theme.colors[props.variant || "input"].disabled};
  }
  @media (max-width: 480px) {
    padding: 0 0.8em;
  }
`;

export const Button = styled.button.attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  min-height: 2.5em;
  height: 2.5em;
  cursor: pointer;
  &:disabled {
    opacity: 0.6;
  }
  border-color: ${(props) => props.theme.colors[props.variant || "input"].color};
  @media (max-width: 700px) {
    height: max-content;
  }
`;

export const MobileButton = styled.button.attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  height: 2.5em;
  width: 3.35em;
  background-color: #00000063;
  align-self: center;
  cursor: pointer;
  border: unset;
  @media (min-width: 481px) {
    display: none;
  }
  &:hover:not(:disabled):not(.static) {
    background-color: #000000a3;
  }
  color: #cccccc !important;
`;

export const AButton = styled.a.attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  height: 2.5em;
  text-decoration: none;
  line-height: 2.5em;
  &:hover:not(:disabled):not(.static) {
    cursor: pointer;
  }
  border-color: ${(props) => props.theme.colors[props.variant || "input"].color};
`;

export const AButtonAlt = styled.a.attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  height: 2.5em;
  text-decoration: none;
  line-height: 2.5em;
  border: unset;
  background-color: #ffffff0a;
  color: #cccccc !important;
  &:hover:not(:disabled):not(.static) {
    background-color: #f9f9f921;
  }
`;

export const Label = styled.label`
  display: block;
  margin-bottom: 10px;

  &[required] {
    ::after {
      content: "  *";
      color: red;
      font-size: 15px;
      font-weight: bolder;
    }
  }
`;

export const NavButton = styled(NavLink).attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  height: 2.5em;
  text-decoration: none;
  line-height: 2.5em;
  border-color: ${(props) => props.theme.colors[props.variant || "input"].color};
`;

export const NavButtonAlt = styled(NavLink).attrs((props) => ({
  className: `${props.active ? "active" : ""} ${props.static ? "static" : ""}`,
}))`
  ${inputStyle}
  height: 2.5em;
  text-decoration: none;
  line-height: 2.5em;
  border: unset;
  background-color: #ffffff0a;
  color: #cccccc !important;

  &:hover:not(:disabled):not(.static) {
    background-color: #f9f9f921;
  }
`;

export const Input = styled.input`
  ${inputStyle}
  height: 2.5em;
`;

export const Radio = styled.input.attrs({ type: "radio" })`
  ${inputStyle}
`;

export const Select = styled.select`
  ${inputStyle}
  height: 2.5em;
  appearance: none;
  > option {
    background-color: ${(props) => props.theme.colors.background};
    color: ${(props) => props.theme.colors.text};
  }
`;

export const SelectAlt = styled.select`
  ${inputStyle}
  height: 2.5em;
  appearance: listbox;
  background-color: #ffffff0a;
  color: #cccccc !important;
  &:hover:not(:disabled):not(.static) {
    background-color: #f9f9f921;
  }

  > option {
    color: black;
  }
`;

export const Textarea = styled.textarea`
  ${inputStyle}
  padding: 1em;

  &:focus::placeholder {
    color: transparent;
  }
`;

export const InputGroup = styled.div`
  display: flex;

  flex-wrap: nowrap;
  > * {
    z-index: 1;
    margin: 0;
  }
  > :not(:last-child) {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }
  > :not(:first-child) {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
    margin-left: -1px;
  }
  > .active:hover {
    z-index: 4;
  }
  > .active {
    z-index: 3;
  }
  > :hover {
    z-index: 2;
  }
  @media (max-width: 480px) {
    ${(props) =>
      !props.fixed &&
      `
	  * {
		  padding-right: 0.3em;
		  padding-left: 0.3em;
		  font-size: 0.85em;
		  svg {
			  padding: 0;
			  margin: 0 0.3em 0 0.3em;
		  }
	  }
  `}
  }
  box-shadow: 3px 3px 1px ${(props) => props.theme.colors.shadow};
  border-radius: 20px;
  width: fit-content;
  height: fit-content;
`;

export const InputGroupAlt = styled(InputGroup)`
  box-shadow: none;
`;

export const Buttons = styled.div`
  display: flex;
  flex-wrap: wrap;
  > * {
    margin-bottom: ${(props) => (props.marginb ? props.marginb : "0.5em")};
    margin-right: 0.5em;
  }
  > :not(:last-child) {
    @media (max-width: 480px) {
      margin-bottom: 0.4em;
      margin-right: 0.3em;
    }
  }
`;

export const CenteredButtons = styled.div`
  display: flex;
  justify-content: center;
  > * {
    width: ${(props) => (props.size ? props.size : "inherit")};
    margin: 0.2em;
  }
`;

export const Highlight = styled.b`
  color: ${(props) => props.theme.colors.highlight.text};
  &:hover:not(:disabled):not(.static) {
    cursor: pointer;
    color: ${(props) => props.theme.colors.highlight.active};
  }
`;

export const CenteredParagraph = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  width: 100%;
`;
CenteredParagraph.Head = styled.h3`
  font-weight: bold;
  font-size: 1.5em;
  text-decoration: underline;
`;
CenteredParagraph.Paragraph = styled.div`
  margin: 0;
  border-top: 2px solid;
  padding: 2em 5em;
  @media (max-width: 720px) {
    padding: 1em;
  }
  border-color: ${(props) => props.theme.colors.accent2};
`;
CenteredParagraph.ParagraphALT = styled.div`
  margin: 0;
  padding: 2em 5em;
  @media (max-width: 720px) {
    padding: 1em;
  }
  b {
    display: block;
  }
  border-color: ${(props) => props.theme.colors.accent2};
`;

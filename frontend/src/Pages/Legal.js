import { Content } from "../Components/Page";
import { usePageTitle } from "../Util/title";

export function Legal() {
  usePageTitle("Legal Notice");
  return (
    <Content>
      <h2>CCP Copyright Notice</h2>
      <p>
        EVE Online and the EVE logo are the registered trademarks of CCP hf. All rights are reserved
        worldwide. All other trademarks are the property of their respective owners. EVE Online, the
        EVE logo, EVE and all associated logos and designs are the intellectual property of CCP hf.
        All artwork, screenshots, characters, vehicles, storylines, world facts or other
        recognizable features of the intellectual property relating to these trademarks are likewise
        the intellectual property of CCP hf. CCP hf. has granted permission to tlaincursions.com to
        use EVE Online and all associated logos and designs for promotional and information purposes
        on its website but does not endorse, and is not in any way affiliated with,
        tlaincursions.com. CCP is in no way responsible for the content on or functioning of this
        website, nor can it be liable for any damage arising from the use of this website.
      </p>
      <p>
        The source code for tlaincursions.com is available under the MIT license. The source code
        and full text for this license can be found{" "}
        <a href="https://github.com/luna-duclos/tla-waitlist">here</a>.
      </p>
    </Content>
  );
}

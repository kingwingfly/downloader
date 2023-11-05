import BtnLink from "@/components/btnLink";
import ConfigIcon from "@/components/configIcon";

export default function Home() {
  return (
    <>
      <BtnLink href="\newTask" content="New a Task" scoll={false} />
      <BtnLink href="\taskList" content="Task List" scoll={false} />
      <ConfigIcon />
    </>
  )
}
import HomeBtn from "@/components/home-btn";
import NewTaskBar from "@/components/new-task-bar";

export default function newTask() {
    return (
        <div className="flex">
            <NewTaskBar />
            <HomeBtn />
        </div>
    )
}